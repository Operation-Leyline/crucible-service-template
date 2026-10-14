use crate::my_service::{CreateItemInput, CreateItemOperation, CreateItemOutput};
use crate::operations::Service;
use crucible_server_runtime_rust::{Result, idempotency};

impl CreateItemOperation for Service {
    async fn create_item(&self, input: CreateItemInput) -> Result<CreateItemOutput> {
        // `Idempotency` (main.rs) has claimed `clientToken`: a retry replays
        // this success instead of running again. Commit before the write a
        // second run would repeat, so a request lost after it is refused
        // on retry rather than creating a second item.
        idempotency::commit().await?;
        match self.items.create(input.name, input.notes) {
            Ok(item) => Ok(CreateItemOutput { item }),
            Err(error) => {
                // `create` wrote nothing, so a retry may run again.
                idempotency::abandon()?;
                Err(error)
            }
        }
    }
}
