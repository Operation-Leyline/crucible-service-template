mod item;

use crate::store::MemoryItems;

#[derive(Default)]
pub struct Service {
    pub(crate) items: MemoryItems,
}
