use super::allocation::{request_resources, AllocationSpec};
use super::builder::{Builder, NodeInput};
use super::prediction::Prediction;
use super::{ActiveControl, GeneratedAction, PlannerCommand};
use crate::adaptive::{
    ActionKind, ActionValue, ActivePlannerContext, CandidateSnapshot, ContinuationDecision,
    ContinuationPolicy, HedgePolicy, InFlightAction,
};

impl Builder<'_> {
    pub(super) fn add_active(&mut self, candidate: &CandidateSnapshot) {
        for active in &candidate.in_flight {
            let Some(context) = self.context.active(active.action_id) else {
                continue;
            };
            self.add_control(candidate, active, context);
            self.add_promotion(candidate, active);
            self.add_hedge(candidate, active, context);
        }
    }

    fn add_control(
        &mut self,
        candidate: &CandidateSnapshot,
        active: &InFlightAction,
        context: &ActivePlannerContext,
    ) {
        let Some(advantage) = context.continuation_advantage_micros else {
            return;
        };
        let decision = ContinuationPolicy::new(50_000, 50_000).decide(advantage);
        self.active_controls.push(ActiveControl {
            action: active.action_id,
            decision,
        });
        if decision == ContinuationDecision::Abort {
            self.push_cancel(candidate, active.action_id, advantage);
        }
    }

    fn add_promotion(&mut self, candidate: &CandidateSnapshot, active: &InFlightAction) {
        let Some(grant) = active.request.promotion() else {
            return;
        };
        if active.cancelling || grant.valid_until_ms < self.snapshot.observed_at_ms {
            return;
        }
        let forecast_kind = ActionKind::FetchWhole {
            maximum_bytes: grant.maximum_bytes,
        };
        let prediction = self.prediction(candidate, &forecast_kind, &active.source);
        let kind = ActionKind::Promote {
            active: active.action_id,
            maximum_bytes: grant.maximum_bytes,
        };
        let input = NodeInput::new(kind, &active.source, prediction, &[]);
        let mut node = self.node(candidate, input);
        node.resources =
            super::super::ResourceCost::new(grant.maximum_bytes, grant.maximum_bytes, 0, 0);
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Promote {
                post: candidate.post.clone(),
                action: active.action_id,
            },
        });
    }

    fn add_hedge(
        &mut self,
        candidate: &CandidateSnapshot,
        active: &InFlightAction,
        context: &ActivePlannerContext,
    ) {
        let Some((input, proof, alternate)) = context.hedge() else {
            return;
        };
        if !HedgePolicy::eligible(input, proof) || alternate == active.source {
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
        node.value = ActionValue::from_net_micros(net_hedge_value(input));
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Hedge {
                primary: active.action_id,
                transfer: allocation,
            },
        });
    }

    fn push_cancel(
        &mut self,
        candidate: &CandidateSnapshot,
        action: crate::ActionId,
        advantage: i64,
    ) {
        let prediction = Prediction {
            forecast: Default::default(),
            uncertainty_bps: 0,
        };
        let input = NodeInput::new(ActionKind::Cancel(action), "local", prediction, &[]);
        let mut node = self.node(candidate, input);
        node.value = ActionValue::from_net_micros(advantage.saturating_neg());
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
