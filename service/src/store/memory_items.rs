use crucible_server_runtime_rust::Result;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::{Mutex, MutexGuard};

use uuid::Uuid;

use crate::my_service::{
    ItemDetails, ResourceNotFoundException, ResourceType, ServiceQuotaExceededException,
};

const MAX_ITEMS: usize = 1_000;

/// Local example storage, ordered by item ID so a page resumes after the
/// last ID it showed. Retries and page tokens are the framework's
/// (`Idempotency` and `PageTokens` in main.rs), so this only keeps items.
#[derive(Default)]
pub struct MemoryItems {
    items: Mutex<BTreeMap<String, ItemDetails>>,
}

/// One page of items, and the ID the next page starts after, if any.
pub struct ItemPage {
    pub items: Vec<ItemDetails>,
    pub last: Option<String>,
}

impl MemoryItems {
    fn lock(&self) -> Result<MutexGuard<'_, BTreeMap<String, ItemDetails>>> {
        self.items
            .lock()
            .map_err(|_| crucible_server_runtime_rust::Error::internal("item store lock poisoned"))
    }

    /// Writes nothing when it fails.
    pub fn create(&self, name: String, notes: Option<String>) -> Result<ItemDetails> {
        let mut items = self.lock()?;
        if items.len() >= MAX_ITEMS {
            return Err(ServiceQuotaExceededException {
                message: "The local example's item limit has been reached.".into(),
                resource_type: ResourceType::Item,
            }
            .into());
        }
        let item = ItemDetails {
            item_id: Uuid::new_v4().to_string(),
            name,
            notes,
        };
        items.insert(item.item_id.clone(), item.clone());
        Ok(item)
    }

    pub fn get(&self, id: &str) -> Result<ItemDetails> {
        self.lock()?.get(id).cloned().ok_or_else(|| missing(id))
    }

    pub fn update(&self, id: &str, name: String, notes: Option<String>) -> Result<ItemDetails> {
        let mut items = self.lock()?;
        let item = items.get_mut(id).ok_or_else(|| missing(id))?;
        item.name = name;
        item.notes = notes;
        Ok(item.clone())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        // Repeated deletes have the same effect and remain safe to retry.
        self.lock()?.remove(id);
        Ok(())
    }

    pub fn list(&self, after: Option<String>, limit: usize) -> Result<ItemPage> {
        let items = self.lock()?;
        let start = after.map_or(Unbounded, Excluded);
        let mut page: Vec<_> = items
            .range((start, Unbounded))
            .take(limit + 1)
            .map(|(_, item)| item.clone())
            .collect();
        let last = if page.len() > limit {
            page.truncate(limit);
            page.last().map(|item| item.item_id.clone())
        } else {
            None
        };
        Ok(ItemPage { items: page, last })
    }
}

fn missing(id: &str) -> crucible_server_runtime_rust::Error {
    ResourceNotFoundException {
        message: "The item does not exist.".into(),
        resource_id: id.into(),
        resource_type: ResourceType::Item,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::MemoryItems;

    #[test]
    fn pages_resume_after_the_last_item_shown() {
        let store = MemoryItems::default();
        for index in 0..3 {
            store.create(format!("Item {index}"), None).unwrap();
        }
        let first = store.list(None, 2).unwrap();
        assert_eq!(first.items.len(), 2);
        let rest = store.list(first.last.clone(), 2).unwrap();
        assert_eq!(rest.items.len(), 1);
        assert!(rest.last.is_none());
        assert!(first.last.unwrap() < rest.items[0].item_id);
    }
}
