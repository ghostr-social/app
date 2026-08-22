pub(crate) struct Streamed {
    pub bytes: u64,
    pub cancelled: bool,
    pub discovered_total: Option<u64>,
}

pub(super) struct StoreProgress {
    pub(super) bytes: u64,
    pub(super) cancelled: bool,
}

impl StoreProgress {
    pub(super) fn complete(bytes: u64) -> Self {
        Self {
            bytes,
            cancelled: false,
        }
    }

    pub(super) fn cancelled(bytes: u64) -> Self {
        Self {
            bytes,
            cancelled: true,
        }
    }
}
