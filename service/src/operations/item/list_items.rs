use crate::my_service::{ListItemsInput, ListItemsOperation, ListItemsOutput};
use crate::operations::Service;
use crucible_server_runtime_rust::Result;

impl ListItemsOperation for Service {
    async fn list_items(&self, input: ListItemsInput) -> Result<ListItemsOutput> {
        self.items.list(
            input.next_token.as_deref(),
            input.max_results.unwrap_or(20) as usize,
        )
    }
}
