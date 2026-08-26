use super::DecisionRecord;
use crate::adaptive::decision::replay::VerifiedWarpReplay;
use crate::adaptive::DecisionReplayStatus;

impl DecisionRecord {
    pub(crate) fn replay_warp(&self) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
        super::replay::warp(self)
    }

    pub(crate) fn replay_warp_search(&self) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
        super::replay::warp_search(self)
    }
}
