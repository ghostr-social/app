use super::{ErrorReason, OriginObservation, OriginOutcome, OriginQuery};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenBodyObservation {
    pub(super) query: OriginQuery,
    observed_at_ms: u64,
    pub(super) outcome: OriginOutcome,
    pub throughput_bps: Option<u64>,
}

impl OpenBodyObservation {
    pub fn success(query: OriginQuery, observed_at_ms: u64) -> Self {
        Self::new(query, observed_at_ms, OriginOutcome::Success)
    }

    pub fn failure(query: OriginQuery, observed_at_ms: u64, reason: ErrorReason) -> Self {
        Self::new(query, observed_at_ms, OriginOutcome::Failure(reason))
    }

    pub fn cancelled(query: OriginQuery, observed_at_ms: u64) -> Self {
        Self::new(query, observed_at_ms, OriginOutcome::Cancelled)
    }

    pub(super) fn transport_observation(&self) -> OriginObservation {
        let mut item = match self.outcome {
            OriginOutcome::Success => {
                OriginObservation::success(self.query.clone(), self.observed_at_ms)
            }
            OriginOutcome::Failure(reason) => {
                OriginObservation::failure(self.query.clone(), self.observed_at_ms, reason)
            }
            OriginOutcome::Cancelled => {
                OriginObservation::cancelled(self.query.clone(), self.observed_at_ms)
            }
        };
        item.throughput_bps = self.throughput_bps;
        item
    }

    fn new(query: OriginQuery, observed_at_ms: u64, outcome: OriginOutcome) -> Self {
        Self {
            query,
            observed_at_ms,
            outcome,
            throughput_bps: None,
        }
    }
}
