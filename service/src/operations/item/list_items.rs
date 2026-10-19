use crate::my_service::{ListItemsInput, ListItemsOperation, ListItemsOutput};
use crate::operations::Service;
use crucible_server_runtime_rust::{Error, Result, page_tokens};

impl ListItemsOperation for Service {
    async fn list_items(&self, input: ListItemsInput) -> Result<ListItemsOutput> {
        // `PageTokens` (main.rs) opened `nextToken` before this ran; what it
        // sealed is the last item ID the previous page showed.
        let after = page_tokens::resume()?
            .map(String::from_utf8)
            .transpose()
            .map_err(|_| Error::internal("a page token sealed a cursor that is not UTF-8"))?;
        let page = self.items.list(after, input.max_results as usize)?;
        Ok(ListItemsOutput {
            items: page.items,
            next_token: page
                .last
                .map(|id| page_tokens::issue(id.as_bytes()))
                .transpose()?,
        })
    }
}
