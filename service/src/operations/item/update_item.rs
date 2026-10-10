use crate::my_service::{UpdateItemInput, UpdateItemOperation, UpdateItemOutput};
use crate::operations::Service;
use crucible_server_runtime_rust::Result;

impl UpdateItemOperation for Service {
    async fn update_item(&self, input: UpdateItemInput) -> Result<UpdateItemOutput> {
        let item = self.items.update(&input.item_id, input.name, input.notes)?;
        Ok(UpdateItemOutput { item })
    }
}
