use super::RecordedWarpDecision;

#[cfg(test)]
mod replay_api_test;

/// A privacy-safe WARP trace whose state, envelope, and internal links were verified.
///
/// This reconstructs the recorded authoritative trace. It does not rerun action generation.
#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarpReplayIntegrity {
    state_hash: String,
    decision_hash: String,
}
