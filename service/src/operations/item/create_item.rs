use crate::contract::{CreateItemOperation, model};
use crate::operations::Service;

impl CreateItemOperation for Service {
    async fn create_item(
        &self,
        input: model::CreateItemInput,
    ) -> crucible::Result<model::CreateItemOutput> {
        let item = self
            .items
            .create(input.name, input.description, input.client_token)?;
        Ok(model::CreateItemOutput { item })
    }
}
