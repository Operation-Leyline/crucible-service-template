use std::ops::Bound::{Excluded, Unbounded};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use uuid::Uuid;

use super::{create_receipt::CreateReceipt, page_cursor::PageCursor, state::State};
use crate::contract::{errors, model};

const MAX_ITEMS: usize = 1_000;
const MAX_RECEIPTS: usize = 10_000;
const MAX_CURSORS: usize = 1_024;
const RECEIPT_TTL: Duration = Duration::from_secs(8 * 60 * 60);
const CURSOR_TTL: Duration = Duration::from_secs(15 * 60);

/// Local example storage. One lock makes each mutation and retry check atomic.
#[derive(Default)]
pub struct MemoryItems {
    state: Mutex<State>,
}

impl MemoryItems {
    fn lock(&self) -> crucible::Result<MutexGuard<'_, State>> {
        self.state
            .lock()
            .map_err(|_| crucible::Error::internal("item store lock poisoned"))
    }

    pub fn create(
        &self,
        name: String,
        description: Option<String>,
        client_token: Option<String>,
    ) -> crucible::Result<model::ItemDetails> {
        let token = client_token.ok_or_else(|| {
            invalid(
                "clientToken",
                "Provide a clientToken for safe create retries.",
            )
        })?;
        let mut state = self.lock()?;
        let now = Instant::now();
        state.receipts.retain(|_, receipt| receipt.expires > now);
        if let Some(receipt) = state.receipts.get(&token) {
            if receipt.name != name || receipt.description != description {
                return Err(errors::ConflictException {
                    message: "This clientToken was already used with different item fields.".into(),
                    resource_id: receipt.item.item_id.clone(),
                    resource_type: model::ResourceType::Item,
                }
                .into());
            }
            return Ok(receipt.item.clone());
        }
        if state.items.len() >= MAX_ITEMS || state.receipts.len() >= MAX_RECEIPTS {
            return Err(errors::ServiceQuotaExceededException {
                message: "The local example's item or create-receipt limit has been reached."
                    .into(),
                resource_type: model::ResourceType::Item,
            }
            .into());
        }
        let item = model::ItemDetails {
            item_id: Uuid::new_v4().to_string(),
            name: name.clone(),
            description: description.clone(),
        };
        state.items.insert(item.item_id.clone(), item.clone());
        state.receipts.insert(
            token,
            CreateReceipt {
                name,
                description,
                item: item.clone(),
                expires: now + RECEIPT_TTL,
            },
        );
        Ok(item)
    }

    pub fn get(&self, id: &str) -> crucible::Result<model::ItemDetails> {
        self.lock()?
            .items
            .get(id)
            .cloned()
            .ok_or_else(|| missing(id))
    }

    pub fn update(
        &self,
        id: &str,
        name: String,
        description: Option<String>,
    ) -> crucible::Result<model::ItemDetails> {
        let mut state = self.lock()?;
        let item = state.items.get_mut(id).ok_or_else(|| missing(id))?;
        item.name = name;
        item.description = description;
        Ok(item.clone())
    }

    pub fn delete(&self, id: &str) -> crucible::Result<()> {
        // Repeated deletes have the same effect and remain safe to retry.
        self.lock()?.items.remove(id);
        Ok(())
    }

    pub fn list(
        &self,
        token: Option<&str>,
        limit: usize,
    ) -> crucible::Result<model::ListItemsOutput> {
        let mut state = self.lock()?;
        let now = Instant::now();
        state.cursors.retain(|_, cursor| cursor.expires > now);
        let after = token
            .map(|token| {
                state
                    .cursors
                    .get(token)
                    .map(|cursor| cursor.after.clone())
                    .ok_or_else(|| {
                        invalid("nextToken", "The pagination token is invalid or expired.")
                    })
            })
            .transpose()?;
        let start = after.map_or(Unbounded, Excluded);
        let mut items: Vec<_> = state
            .items
            .range((start, Unbounded))
            .take(limit + 1)
            .map(|(_, item)| item.clone())
            .collect();
        let next_token = if items.len() > limit {
            items.truncate(limit);
            if state.cursors.len() >= MAX_CURSORS {
                let oldest = state
                    .cursors
                    .iter()
                    .min_by_key(|(_, cursor)| cursor.expires)
                    .map(|(token, _)| token.clone());
                if let Some(token) = oldest {
                    state.cursors.remove(&token);
                }
            }
            // Random references hide the cursor and reject client-constructed values.
            let token = format!("v1-{}", Uuid::new_v4());
            state.cursors.insert(
                token.clone(),
                PageCursor {
                    after: items
                        .last()
                        .expect("positive modeled page size")
                        .item_id
                        .clone(),
                    expires: now + CURSOR_TTL,
                },
            );
            Some(token)
        } else {
            None
        };
        Ok(model::ListItemsOutput { items, next_token })
    }
}

fn missing(id: &str) -> crucible::Error {
    errors::ResourceNotFoundException {
        message: "The item does not exist.".into(),
        resource_id: id.into(),
        resource_type: model::ResourceType::Item,
    }
    .into()
}

fn invalid(field: &str, message: &str) -> crucible::Error {
    errors::ValidationException {
        message: message.into(),
        reason: model::ValidationExceptionReason::FieldValidationFailed,
        field_list: Some(vec![model::ValidationExceptionField {
            name: field.into(),
            message: message.into(),
        }]),
    }
    .into()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::MemoryItems;
    use crate::contract::errors;

    #[test]
    fn concurrent_create_retries_have_one_result() {
        let store = Arc::new(MemoryItems::default());
        let requests: Vec<_> = (0..12)
            .map(|_| {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    store
                        .create("Concurrent".into(), None, Some("same-token".into()))
                        .unwrap()
                        .item_id
                })
            })
            .collect();
        let ids: Vec<_> = requests
            .into_iter()
            .map(|request| request.join().unwrap())
            .collect();
        assert!(ids.iter().all(|id| id == &ids[0]));
        assert_eq!(store.list(None, 100).unwrap().items.len(), 1);
    }

    #[test]
    fn expired_pagination_references_are_rejected() {
        let store = MemoryItems::default();
        for index in 0..2 {
            store
                .create(
                    format!("Item {index}"),
                    None,
                    Some(format!("token-{index}")),
                )
                .unwrap();
        }
        let token = store.list(None, 1).unwrap().next_token.unwrap();
        store
            .state
            .lock()
            .unwrap()
            .cursors
            .get_mut(&token)
            .unwrap()
            .expires = Instant::now() - Duration::from_secs(1);
        let error = store.list(Some(&token), 1).unwrap_err();
        let validation = error.downcast_ref::<errors::ValidationException>().unwrap();
        assert_eq!(validation.field_list.as_ref().unwrap()[0].name, "nextToken");
    }
}
