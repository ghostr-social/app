use super::*;

impl NetworkTokenBucket {
    pub(crate) const fn from_replay(parts: (u64, u64, u64, u64, u64, u64)) -> Self {
        Self {
            capacity: parts.0,
            refill_per_second: parts.1,
            tokens: parts.2,
            updated_at_ms: parts.3,
            refill_milli_bytes: parts.4,
            debt_bytes: parts.5,
        }
    }
}
