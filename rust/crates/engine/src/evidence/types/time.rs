use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceTime {
    pub observed_at_ms: u64,
    pub order: u64,
}

impl EvidenceTime {
    pub const fn ordered(observed_at_ms: u64, order: u64) -> Self {
        Self {
            observed_at_ms,
            order,
        }
    }

    pub(crate) fn is_after(self, other: Self) -> bool {
        match (self.order, other.order) {
            (current, previous) if current > 0 && previous > 0 => current > previous,
            _ => self.observed_at_ms > other.observed_at_ms,
        }
    }
}

impl From<u64> for EvidenceTime {
    fn from(observed_at_ms: u64) -> Self {
        Self::ordered(observed_at_ms, 0)
    }
}
