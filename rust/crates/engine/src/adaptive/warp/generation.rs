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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HlsGenerationPolicy {
    LegacyWholeStage,
    BoundedObjectCursor,
}

pub(crate) struct WarpGenerationInput<'a> {
    snapshot: &'a PlayabilitySnapshot,
    base: &'a AllocationPlan,
    origins: &'a OriginModel,
    context: &'a PlannerContext,
}

impl<'a> WarpGenerationInput<'a> {
    pub(crate) const fn new(
        snapshot: &'a PlayabilitySnapshot,
        base: &'a AllocationPlan,
        origins: &'a OriginModel,
        context: &'a PlannerContext,
    ) -> Self {
        Self {
            snapshot,
            base,
            origins,
            context,
        }
    }
}

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
        cursor: crate::adaptive::HlsObjectCursor,
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
        Self::generate_with_policy(
            WarpGenerationInput::new(snapshot, base, origins, context),
            HlsGenerationPolicy::BoundedObjectCursor,
        )
    }

    pub(crate) fn generate_with_policy(
        input: WarpGenerationInput<'_>,
        hls_policy: HlsGenerationPolicy,
    ) -> GeneratedActions {
        let mut builder =
            builder::Builder::new(input.snapshot, input.base, input.origins, input.context);
        for candidate in &input.snapshot.candidates {
            if candidate.retrieval_eligible {
                builder.add_candidate(candidate);
            }
        }
        for candidate in &input.snapshot.hls_candidates {
            hls::add(&mut builder, candidate, hls_policy);
        }
        builder.add_detached_active();
        builder.finish()
    }
}
