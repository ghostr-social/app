use crate::media_timeline::StartupFootprint;
use crate::{ByteRange, PostId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NextReserveInfeasibility {
    CurrentUnprotected,
    NoLiveOrigin,
    PolicyDenied,
    NoTransferBudget,
    NoStorageCapacity,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ControlMode {
    Emergency,
    Safety,
    #[default]
    Normal,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReserveCandidateState {
    #[default]
    Unprepared,
    Ready { startup: StartupFootprint },
    Structural { startup: StartupFootprint },
    InFlight,
    Probing,
    Preparing { ranges: Vec<ByteRange> },
    Planned { ranges: Vec<ByteRange> },
    Infeasible { reason: NextReserveInfeasibility },
    HlsReady,
    HlsStructural,
    HlsInFlight { stage: super::super::HlsBootstrapStage },
    HlsPending { stage: super::super::HlsBootstrapStage },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReserveCandidateKind {
    #[default]
    Progressive,
    Hls,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReserveCandidateEvidence {
    pub post: PostId,
    #[serde(default)]
    pub kind: ReserveCandidateKind,
    pub state: ReserveCandidateState,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReadyReserveEvidence {
    pub target: usize,
    pub ready: usize,
    pub structural: usize,
    pub protected: usize,
    pub recovery_horizon_ms: u64,
    pub underflow_risk_bps: u16,
    pub ready_coverage_ms: u64,
    pub candidates: Vec<ReserveCandidateEvidence>,
}

impl ReadyReserveEvidence {
    pub fn ordered_ready(&self) -> usize {
        self.candidates
            .iter()
            .take_while(|item| {
                matches!(
                    item.state,
                    ReserveCandidateState::Ready { .. } | ReserveCandidateState::HlsReady
                )
            })
            .count()
    }

    pub fn ordered_target_satisfied(&self) -> bool {
        self.ordered_ready() >= self.target
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum NextReserveEvidence {
    #[default]
    NotApplicable,
    Ready { post: PostId, startup: StartupFootprint },
    Structural { post: PostId, startup: StartupFootprint },
    InFlight { post: PostId },
    Granted { post: PostId, range: ByteRange },
    Infeasible { post: PostId, reason: NextReserveInfeasibility },
    HlsReady { post: PostId },
    HlsStructural { post: PostId },
    HlsInFlight { post: PostId, stage: super::super::HlsBootstrapStage },
    HlsPending { post: PostId, stage: super::super::HlsBootstrapStage },
}
