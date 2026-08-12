use super::DEFAULT_RESERVE_BYTES;

/// One live storage-capacity reading used by delivery admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacitySnapshot {
    limit_bytes: u64,
    used_bytes: u64,
}

impl CapacitySnapshot {
    pub(crate) fn new(limit_bytes: u64, used_bytes: u64) -> Self {
        Self {
            limit_bytes,
            used_bytes,
        }
    }

    pub fn limit_bytes(self) -> u64 {
        self.limit_bytes
    }

    pub fn used_bytes(self) -> u64 {
        self.used_bytes
    }
}

/// The two ceilings the store obeys.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// What the user configured; `u64::MAX` means "no budget of its own".
    pub budget: u64,
    /// Free space that must survive whatever the store does.
    pub reserve: u64,
}

impl Limits {
    /// `budget` against the default device reserve.
    pub(super) fn budget(budget: u64) -> Self {
        Self {
            budget,
            reserve: DEFAULT_RESERVE_BYTES,
        }
    }
}
