use crate::contract::{UpdateItemOperation, model};
use crate::operations::Service;

impl UpdateItemOperation for Service {
    async fn update_item(
        &self,
        input: model::UpdateItemInput,
    ) -> crucible::Result<model::UpdateItemOutput> {
        let item = self
            .items
            .update(&input.item_id, input.name, input.description)?;
        Ok(model::UpdateItemOutput { item })
    }
}
