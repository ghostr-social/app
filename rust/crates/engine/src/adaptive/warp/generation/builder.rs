use super::allocation::request_resources;
use super::prediction::{predict, Prediction};
use super::value;
use super::{
    ActiveControl, GeneratedAction, GeneratedActions, PlannerCommand, PlannerContext,
    PromotionGenerationPolicy, WarpGenerationInput, WarpGenerationPolicies,
};
use crate::adaptive::{AllocationPlan, CandidateSnapshot, PlayabilitySnapshot};

mod admission;
mod transfer;
pub(super) use transfer::TransferInput;

pub(super) struct Builder<'a> {
    pub(super) snapshot: &'a PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) origins: &'a crate::origin_model::OriginModel,
    pub(super) context: &'a PlannerContext,
    pub(super) actions: Vec<GeneratedAction>,
    pub(super) active_controls: Vec<ActiveControl>,
    pub(super) promotion_policy: PromotionGenerationPolicy,
    pub(super) generation_policies: WarpGenerationPolicies,
    next_id: u16,
}

pub(super) struct NodeInput<'a> {
    pub kind: super::super::ActionKind,
    pub source: &'a str,
    pub prediction: Prediction,
    pub requires: &'a [u16],
    pub intent: crate::origin_model::OriginAdmissionIntent,
}

impl<'a> NodeInput<'a> {
    pub(super) const fn new(
        kind: super::super::ActionKind,
        source: &'a str,
        prediction: Prediction,
        requires: &'a [u16],
    ) -> Self {
        Self {
            kind,
            source,
            prediction,
            requires,
            intent: crate::origin_model::OriginAdmissionIntent::Delivery,
        }
    }

    pub(super) const fn optional_exploration(mut self) -> Self {
        self.intent = crate::origin_model::OriginAdmissionIntent::OptionalExploration;
        self
    }

    pub(super) const fn with_intent(
        mut self,
        intent: crate::origin_model::OriginAdmissionIntent,
    ) -> Self {
        self.intent = intent;
        self
    }
}

impl<'a> Builder<'a> {
    pub(super) fn new(
        input: WarpGenerationInput<'a>,
        policies: WarpGenerationPolicies,
    ) -> Self {
        Self {
            snapshot: input.snapshot,
            base: input.base,
            origins: input.origins,
            context: input.context,
            actions: Vec::new(),
            active_controls: Vec::new(),
            promotion_policy: policies.promotion,
            generation_policies: policies,
            next_id: 0,
        }
    }

    pub(super) fn node(
        &mut self,
        candidate: &CandidateSnapshot,
        input: NodeInput<'_>,
    ) -> super::super::ActionNode {
        let id = self.next_action_id();
        let forecast = super::quality::incremental(candidate, self.context, input.prediction);
        let value = value::score(candidate, &input.kind, input.prediction, self.base.mode);
        let (value, intent) = self.generation_policies.apply_origin(value, input.intent);
        super::super::ActionNode::new(id, candidate.post.clone(), input.kind, value)
            .with_origin(input.source)
            .with_request_profile(input.prediction.request_profile)
            .with_origin_admission_intent(intent)
            .with_forecast(forecast)
            .requiring(input.requires)
    }

    pub(super) fn local_node(
        &mut self,
        post: &crate::PostId,
        kind: super::super::ActionKind,
        value: super::super::ActionValue,
    ) -> super::super::ActionNode {
        super::super::ActionNode::new(self.next_action_id(), post.clone(), kind, value)
    }

    pub(super) fn next_action_id(&mut self) -> u16 {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("planner action id exhausted");
        self.next_id
    }

    pub(super) fn prediction(
        &self,
        candidate: &CandidateSnapshot,
        kind: &super::super::ActionKind,
        source: &str,
    ) -> Prediction {
        predict(super::prediction::PredictionInput {
            model: self.origins,
            snapshot: self.snapshot,
            base: self.base,
            candidate,
            action: kind,
            source,
            concurrency: self
                .context
                .request_occupancy()
                .authority_count(source)
                .saturating_add(1),
            mode: self.base.mode,
            direct_playback_blocked: self.direct_playback_blocked(candidate),
            network_class: self.context.network_class(),
        })
    }

    pub(super) fn direct_playback_blocked(&self, candidate: &CandidateSnapshot) -> bool {
        self.context
            .candidate(&candidate.post)
            .is_some_and(|item| item.capability.blocks_direct_playback())
    }

    pub(super) fn action_id(
        &self,
        candidate: &CandidateSnapshot,
        matches: impl Fn(&super::super::ActionKind) -> bool,
    ) -> Option<u16> {
        self.actions
            .iter()
            .find(|item| item.node.post == candidate.post && matches(&item.node.kind))
            .map(|item| item.node.id)
    }

    pub(super) fn contains(
        &self,
        candidate: &CandidateSnapshot,
        kind: &super::super::ActionKind,
    ) -> bool {
        self.actions
            .iter()
            .any(|item| item.node.post == candidate.post && &item.node.kind == kind)
    }

    pub(super) fn finish(self) -> GeneratedActions {
        let ladders = super::ladders::build(self.snapshot, &self.actions, self.context);
        GeneratedActions {
            actions: self.actions,
            ladders,
            active_controls: self.active_controls,
        }
    }
}
