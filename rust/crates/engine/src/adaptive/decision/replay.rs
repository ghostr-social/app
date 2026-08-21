use super::{
    RecordedResourcePrices, RecordedWarpAction, RecordedWarpDecision, RecordedWarpReserve,
    RecordedWarpSearch,
};

/// A privacy-safe WARP trace whose state, envelope, and internal links were verified.
///
/// This reconstructs the recorded authoritative trace. It does not rerun action generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedWarpReplay {
    sequence: u64,
    integrity: WarpReplayIntegrity,
    decision: RecordedWarpDecision,
}

impl VerifiedWarpReplay {
    pub(in crate::adaptive::decision) fn new(
        sequence: u64,
        state_hash: String,
        decision_hash: String,
        decision: RecordedWarpDecision,
    ) -> Self {
        Self {
            sequence,
            integrity: WarpReplayIntegrity {
                state_hash,
                decision_hash,
            },
            decision,
        }
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn integrity(&self) -> &WarpReplayIntegrity {
        &self.integrity
    }

    pub const fn decision(&self) -> &RecordedWarpDecision {
        &self.decision
    }

    pub fn selected(&self) -> Option<&RecordedWarpAction> {
        self.decision.selected.as_ref()
    }

    pub const fn search(&self) -> &RecordedWarpSearch {
        &self.decision.search
    }

    pub const fn common_random_seed(&self) -> u64 {
        self.decision.search.common_random_seed
    }

    pub const fn prices(&self) -> RecordedResourcePrices {
        self.decision.prices
    }

    pub const fn reserve(&self) -> RecordedWarpReserve {
        self.decision.reserve
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarpReplayIntegrity {
    state_hash: String,
    decision_hash: String,
}

impl WarpReplayIntegrity {
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    pub fn decision_hash(&self) -> &str {
        &self.decision_hash
    }
}
