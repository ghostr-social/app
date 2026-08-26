use super::{ErrorReason, OriginQuery};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OriginOutcome {
    Success,
    Failure(ErrorReason),
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginObservation {
    pub query: OriginQuery,
    pub observed_at_ms: u64,
    pub outcome: OriginOutcome,
    pub range_compliant: Option<bool>,
    pub ttfb_ms: Option<u64>,
    pub throughput_bps: Option<u64>,
}

impl OriginObservation {
    pub fn success(query: OriginQuery, observed_at_ms: u64) -> Self {
        Self::new(query, observed_at_ms, OriginOutcome::Success)
    }

    pub fn failure(query: OriginQuery, observed_at_ms: u64, reason: ErrorReason) -> Self {
        Self::new(query, observed_at_ms, OriginOutcome::Failure(reason))
    }

    pub fn cancelled(query: OriginQuery, observed_at_ms: u64) -> Self {
        Self::new(query, observed_at_ms, OriginOutcome::Cancelled)
    }

    pub fn with_ttfb_ms(mut self, value: u64) -> Self {
        self.ttfb_ms = Some(value.max(1));
        self
    }

    fn new(query: OriginQuery, observed_at_ms: u64, outcome: OriginOutcome) -> Self {
        Self {
            query,
            observed_at_ms,
            outcome,
            range_compliant: None,
            ttfb_ms: None,
            throughput_bps: None,
        }
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "observation/test_support.rs"]
mod test_support;
