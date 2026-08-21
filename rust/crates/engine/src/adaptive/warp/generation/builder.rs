use super::allocation::resources;
use super::prediction::{predict, Prediction};
use super::value;
use super::{ActiveControl, GeneratedAction, GeneratedActions, PlannerCommand, PlannerContext};
use crate::adaptive::{Allocation, AllocationPlan, CandidateSnapshot, PlayabilitySnapshot};
use crate::origin_model::OriginModel;

pub(super) fn build(
    snapshot: &PlayabilitySnapshot,
    base: &AllocationPlan,
    origins: &OriginModel,
    context: &PlannerContext,
) -> GeneratedActions {
    let mut builder = Builder::new(snapshot, base, origins, context);
    for candidate in snapshot
        .candidates
        .iter()
        .filter(|item| item.retrieval_eligible)
    {
        builder.add_candidate(candidate);
    }
    builder.finish()
}

pub(super) struct Builder<'a> {
    pub(super) snapshot: &'a PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) origins: &'a OriginModel,
    pub(super) context: &'a PlannerContext,
    pub(super) actions: Vec<GeneratedAction>,
    pub(super) active_controls: Vec<ActiveControl>,
    next_id: u16,
}

impl<'a> Builder<'a> {
    fn new(
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
            actions: Vec::new(),
            active_controls: Vec::new(),
            next_id: 0,
        }
    }

    pub(super) fn push_transfer(
        &mut self,
        candidate: &CandidateSnapshot,
        kind: super::super::ActionKind,
        allocation: Allocation,
        requires: &[u16],
    ) -> u16 {
        let prediction = self.prediction(candidate, &kind, &allocation.source);
        let mut node = self.node(
            candidate,
            kind.clone(),
            &allocation.source,
            prediction,
            requires,
        );
        node.resources = resources(&kind);
        let id = node.id;
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Transfer(allocation),
        });
        id
    }

    pub(super) fn node(
        &mut self,
        candidate: &CandidateSnapshot,
        kind: super::super::ActionKind,
        source: &str,
        prediction: Prediction,
        requires: &[u16],
    ) -> super::super::ActionNode {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("planner action id exhausted");
        super::super::ActionNode::new(
            self.next_id,
            candidate.post.clone(),
            kind.clone(),
            value::score(candidate, &kind, prediction, self.base.mode),
        )
        .with_origin(source)
        .with_forecast(prediction.forecast)
        .requiring(requires)
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
            candidate,
            action: kind,
            source,
            concurrency: self
                .context
                .request_occupancy()
                .authority_count(source)
                .saturating_add(1),
            mode: self.base.mode,
        })
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

    fn finish(self) -> GeneratedActions {
        let ladders = super::ladders::build(self.snapshot, &self.actions, self.context);
        GeneratedActions {
            actions: self.actions,
            ladders,
            active_controls: self.active_controls,
        }
    }
}
