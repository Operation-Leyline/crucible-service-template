use crate::my_service::{DeleteItemInput, DeleteItemOperation};
use crate::operations::Service;
use crucible_server_runtime_rust::Result;

impl DeleteItemOperation for Service {
    async fn delete_item(&self, input: DeleteItemInput) -> Result<()> {
        self.items.delete(&input.item_id)
    }
}
