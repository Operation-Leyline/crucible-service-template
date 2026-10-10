use std::time::Instant;

use crate::my_service::ItemDetails;

pub(super) struct CreateReceipt {
    pub name: String,
    pub notes: Option<String>,
    pub item: ItemDetails,
    pub expires: Instant,
}
