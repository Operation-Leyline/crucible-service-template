use std::time::Instant;

pub(super) struct PageCursor {
    pub after: String,
    pub expires: Instant,
}
