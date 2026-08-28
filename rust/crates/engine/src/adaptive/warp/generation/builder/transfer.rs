use super::{Builder, NodeInput};
use crate::adaptive::{Allocation, CandidateSnapshot};
use crate::origin_model::OriginAdmissionIntent;

pub(in crate::adaptive::warp::generation) struct TransferInput<'a> {
    kind: super::super::super::ActionKind,
    allocation: Allocation,
    requires: &'a [u16],
    intent: OriginAdmissionIntent,
}

impl<'a> TransferInput<'a> {
    pub(in crate::adaptive::warp::generation) const fn delivery(
        kind: super::super::super::ActionKind,
        allocation: Allocation,
        requires: &'a [u16],
    ) -> Self {
        Self {
            kind,
            allocation,
            requires,
            intent: OriginAdmissionIntent::Delivery,
        }
    }

    pub(in crate::adaptive::warp::generation) const fn optional_exploration(
        kind: super::super::super::ActionKind,
        allocation: Allocation,
        requires: &'a [u16],
    ) -> Self {
        Self {
            kind,
            allocation,
            requires,
            intent: OriginAdmissionIntent::OptionalExploration,
        }
    }
}

impl Builder<'_> {
    pub(in crate::adaptive::warp::generation) fn push_transfer(
        &mut self,
        candidate: &CandidateSnapshot,
        input: TransferInput<'_>,
    ) -> Option<u16> {
        let source = input.allocation.source.as_str();
        if !self.source_admitted(candidate, &input.kind, source, input.intent) {
            return None;
        }
        let prediction = self.prediction(candidate, &input.kind, source);
        let node_input = NodeInput::new(input.kind, source, prediction, input.requires);
        let mut node = self.node(candidate, node_input.with_intent(input.intent));
        node.resources = super::request_resources(input.allocation.request);
        node = node.with_request(input.allocation.request);
        let id = node.id;
        self.actions.push(super::GeneratedAction {
            node,
            command: super::PlannerCommand::Transfer(input.allocation),
        });
        Some(id)
    }
}
