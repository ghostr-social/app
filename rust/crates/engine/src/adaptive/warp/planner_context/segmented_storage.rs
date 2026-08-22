use serde::{Deserialize, Serialize};

use super::PlannerContext;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SegmentedStorageBudget(u64);

impl SegmentedStorageBudget {
    pub const fn new(available_bytes: u64) -> Self {
        Self(available_bytes)
    }

    pub const fn available_bytes(self) -> u64 {
        self.0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub(in crate::adaptive::warp) const fn consume(self, bytes: u64) -> Option<Self> {
        match self.0.checked_sub(bytes) {
            Some(remaining) => Some(Self(remaining)),
            None => None,
        }
    }
}

impl PlannerContext {
    pub fn with_segmented_storage_available_bytes(mut self, available_bytes: u64) -> Self {
        self.segmented_storage_available_bytes = SegmentedStorageBudget::new(available_bytes);
        self
    }

    pub(in crate::adaptive::warp) const fn segmented_storage_budget(
        &self,
    ) -> SegmentedStorageBudget {
        self.segmented_storage_available_bytes
    }
}
