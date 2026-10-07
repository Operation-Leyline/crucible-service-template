use std::collections::{BTreeMap, HashMap};

use super::{create_receipt::CreateReceipt, page_cursor::PageCursor};
use crate::contract::model::ItemDetails;

#[derive(Default)]
pub(super) struct State {
    pub items: BTreeMap<String, ItemDetails>,
    pub receipts: HashMap<String, CreateReceipt>,
    pub cursors: HashMap<String, PageCursor>,
}
