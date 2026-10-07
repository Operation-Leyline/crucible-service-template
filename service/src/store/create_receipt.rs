use std::time::Instant;

use crate::contract::model::ItemDetails;

pub(super) struct CreateReceipt {
    pub name: String,
    pub description: Option<String>,
    pub item: ItemDetails,
    pub expires: Instant,
}
