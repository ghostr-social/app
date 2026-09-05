use super::allocation::{request_resources, AllocationSpec};
use super::builder::{Builder, NodeInput};
use super::{ActiveControl, GeneratedAction, PlannerCommand};
use crate::adaptive::{
    ActionKind, ActionValue, ActivePlannerContext, CandidateSnapshot, ContinuationDecision,
    ContinuationPolicy, HedgePolicy, InFlightAction,
};

mod promotion;

impl Builder<'_> {
    pub(super) fn add_active(&mut self, candidate: &CandidateSnapshot) {
        for active in &candidate.in_flight {
            if active.cancelling {
                continue;
            }
            let Some(context) = self.context.active(active.action_id) else {
                continue;
            };
            self.add_control(&candidate.post, active.action_id, context);
            self.add_promotion(candidate, active);
            self.add_hedge(candidate, active, context);
        }
    }

    pub(super) fn add_detached_active(&mut self) {
        let detached: Vec<_> = self
            .context
            .active_contexts()
            .filter(|context| !context.cancelling() && !self.represents(context.action))
            .cloned()
            .collect();
        for context in detached {
            self.add_control(context.post(), context.action, &context);
        }
    }

    fn represents(&self, action: crate::ActionId) -> bool {
        self.snapshot
            .candidates
            .iter()
            .filter(|candidate| candidate.retrieval_eligible)
            .flat_map(|candidate| &candidate.in_flight)
            .any(|active| active.action_id == action)
    }

    fn add_control(
        &mut self,
        post: &crate::PostId,
        action: crate::ActionId,
        context: &ActivePlannerContext,
    ) {
        let Some(advantage) = context.continuation_advantage_micros else {
            return;
        };
        let decision = ContinuationPolicy::new(50_000, 50_000).decide(advantage);
        self.active_controls
            .push(ActiveControl { action, decision });
        if decision == ContinuationDecision::Abort {
            self.push_cancel(post, action, advantage);
        }
    }

    fn add_hedge(
        &mut self,
        candidate: &CandidateSnapshot,
        active: &InFlightAction,
        context: &ActivePlannerContext,
    ) {
        if !self.generation_policies.hedging || !self.permits_request(candidate) {
            return;
        }
        let Some((input, proof, alternate)) = context.hedge() else {
            return;
        };
        if !exact_hedge(input, active)
            || !HedgePolicy::eligible(input, proof)
            || alternate == active.source
            || !self.source_admitted(
                candidate,
                &input.action,
                alternate,
                crate::origin_model::OriginAdmissionIntent::Delivery,
            )
        {
            return;
        }
        let allocation =
            self.allocation(candidate, AllocationSpec::hedge(active.request, alternate));
        let kind = ActionKind::Hedge {
            primary: active.action_id,
            alternate: alternate.to_owned(),
        };
        let prediction = self.prediction(candidate, &input.action, alternate);
        let node_input = NodeInput::new(kind, alternate, prediction, &[]);
        let mut node = self.node(candidate, node_input);
        node.resources = request_resources(allocation.request);
        node = node.with_request(allocation.request);
        node.value = ActionValue::from_net_micros(net_hedge_value(input));
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Hedge {
                primary: active.action_id,
                transfer: allocation,
            },
        });
    }

    fn push_cancel(&mut self, post: &crate::PostId, action: crate::ActionId, advantage: i64) {
        let value = ActionValue::from_net_micros(advantage.saturating_neg());
        let node = self.local_node(post, ActionKind::Cancel(action), value);
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Cancel(action),
        });
    }
}

fn net_hedge_value(input: &crate::adaptive::HedgeInput) -> i64 {
    input
        .loss_reduction_micros
        .saturating_sub(input.duplicate_cost_micros)
        .min(i64::MAX as u64) as i64
}

fn exact_hedge(input: &crate::adaptive::HedgeInput, active: &InFlightAction) -> bool {
    input.primary == active.action_id
        && input.maximum_network_bytes == active.request.reserved_network_bytes()
        && request_matches(&input.action, active.request)
}

fn request_matches(action: &ActionKind, request: crate::adaptive::RetrievalRequest) -> bool {
    match (action, request) {
        (
            ActionKind::Prefix(action) | ActionKind::Tail(action) | ActionKind::FetchRange(action),
            crate::adaptive::RetrievalRequest::FetchRange { bytes, .. },
        ) => *action == bytes,
        (
            ActionKind::FetchWhole { maximum_bytes },
            crate::adaptive::RetrievalRequest::FetchWhole { contract, .. },
        ) => *maximum_bytes == contract.maximum_bytes(),
        _ => false,
    }
}
