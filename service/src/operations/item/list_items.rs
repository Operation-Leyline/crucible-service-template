use crate::contract::{ListItemsOperation, model};
use crate::operations::Service;

impl ListItemsOperation for Service {
    async fn list_items(
        &self,
        input: model::ListItemsInput,
    ) -> crucible::Result<model::ListItemsOutput> {
        self.items.list(
            input.next_token.as_deref(),
            input.max_results.unwrap_or(20) as usize,
        )
    }
}
