use super::{VerifiedWarpReplay, WarpReplayIntegrity};
use crate::adaptive::{
    RecordedResourcePrices, RecordedWarpAction, RecordedWarpDecision, RecordedWarpReserve,
    RecordedWarpSearch,
};

impl VerifiedWarpReplay {
    pub(crate) const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) const fn integrity(&self) -> &WarpReplayIntegrity {
        &self.integrity
    }

    pub(crate) const fn decision(&self) -> &RecordedWarpDecision {
        &self.decision
    }

    pub(crate) fn selected(&self) -> Option<&RecordedWarpAction> {
        self.decision.selected.as_ref()
    }

    pub(crate) const fn search(&self) -> &RecordedWarpSearch {
        &self.decision.search
    }

    pub(crate) const fn common_random_seed(&self) -> u64 {
        self.decision.search.common_random_seed
    }

    pub(crate) const fn prices(&self) -> RecordedResourcePrices {
        self.decision.prices
    }

    pub(crate) fn reserve(&self) -> RecordedWarpReserve {
        self.decision.reserve.clone()
    }
}

impl WarpReplayIntegrity {
    pub(crate) fn state_hash(&self) -> &str {
        &self.state_hash
    }

    pub(crate) fn decision_hash(&self) -> &str {
        &self.decision_hash
    }
}
