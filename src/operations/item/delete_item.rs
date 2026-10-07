use crate::contract::{DeleteItemOperation, model};
use crate::operations::Service;

impl DeleteItemOperation for Service {
    async fn delete_item(&self, input: model::DeleteItemInput) -> crucible::Result<()> {
        self.items.delete(&input.item_id)
    }
}
