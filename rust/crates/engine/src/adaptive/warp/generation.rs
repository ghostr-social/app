mod active;
mod allocation;
mod builder;
mod candidate;
mod hls;
mod hls_prediction;
mod ladders;
mod prediction;
mod quality;
mod value;

use super::{ActionNode, ContinuationDecision, PlannerContext, TransformKind};
use crate::adaptive::{
    Allocation, AllocationPlan, PlayabilitySnapshot, PromotionGrant, RetrievalLadder,
};
use crate::origin_model::OriginModel;
use crate::{ActionId, PostId};

#[derive(Clone, Debug, PartialEq)]
pub enum PlannerCommand {
    ProbeHead {
        post: PostId,
        source: String,
        authority: crate::adaptive::PreemptionAuthority,
    },
    Transfer(Allocation),
    FetchHlsBootstrap {
        post: PostId,
        stage: crate::adaptive::HlsBootstrapStage,
        source: String,
        maximum_bytes: u64,
        committed_until_ms: u64,
    },
    Promote {
        post: PostId,
        action: ActionId,
        source: String,
        grant: PromotionGrant,
    },
    Transform {
        post: PostId,
        kind: TransformKind,
    },
    Hedge {
        primary: ActionId,
        transfer: Allocation,
    },
    Cancel(ActionId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedAction {
    pub node: ActionNode,
    pub command: PlannerCommand,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateRetrievalLadder {
    pub post: PostId,
    pub frontier: RetrievalLadder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveControl {
    pub action: ActionId,
    pub decision: ContinuationDecision,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedActions {
    pub actions: Vec<GeneratedAction>,
    pub ladders: Vec<CandidateRetrievalLadder>,
    pub active_controls: Vec<ActiveControl>,
}

pub struct WarpActionGenerator;

impl WarpActionGenerator {
    pub fn generate(
        snapshot: &PlayabilitySnapshot,
        base: &AllocationPlan,
        origins: &OriginModel,
        context: &PlannerContext,
    ) -> GeneratedActions {
        builder::build(snapshot, base, origins, context)
    }
}
