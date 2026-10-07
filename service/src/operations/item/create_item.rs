use crate::my_service::{CreateItemInput, CreateItemOperation, CreateItemOutput};
use crate::operations::Service;
use crucible_server_runtime_rust::Result;

impl CreateItemOperation for Service {
    async fn create_item(&self, input: CreateItemInput) -> Result<CreateItemOutput> {
        let item = self
            .items
            .create(input.name, input.description, input.client_token)?;
        Ok(CreateItemOutput { item })
    }
}
