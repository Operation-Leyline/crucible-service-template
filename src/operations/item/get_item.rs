use crate::contract::{GetItemOperation, model};
use crate::operations::Service;

impl GetItemOperation for Service {
    async fn get_item(&self, input: model::GetItemInput) -> crucible::Result<model::GetItemOutput> {
        Ok(model::GetItemOutput {
            item: self.items.get(&input.item_id)?,
        })
    }
}
