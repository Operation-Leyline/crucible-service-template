use crate::my_service::{GetItemInput, GetItemOperation, GetItemOutput};
use crate::operations::Service;
use crucible_server_runtime_rust::Result;

impl GetItemOperation for Service {
    async fn get_item(&self, input: GetItemInput) -> Result<GetItemOutput> {
        Ok(GetItemOutput {
            item: self.items.get(&input.item_id)?,
        })
    }
}
