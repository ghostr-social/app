#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WholeBodyExhaustion {
    maximum_bytes: u64,
    observed_bytes: u64,
}

impl WholeBodyExhaustion {
    pub const fn new(maximum_bytes: u64, observed_bytes: u64) -> Option<Self> {
        if observed_bytes <= maximum_bytes {
            return None;
        }
        Some(Self {
            maximum_bytes,
            observed_bytes,
        })
    }

    pub const fn maximum_bytes(self) -> u64 {
        self.maximum_bytes
    }

    pub const fn observed_bytes(self) -> u64 {
        self.observed_bytes
    }
}

impl PlannerContext {
    pub fn with_whole_body_exhaustion(
        mut self,
        post: &PostId,
        exhaustion: WholeBodyExhaustion,
    ) -> Self {
        if let Some(candidate) = self.candidates.get_mut(post) {
            candidate.whole_body_exhaustion = Some(exhaustion);
        }
        self
    }
}
use super::super::PlannerContext;
use crate::PostId;
