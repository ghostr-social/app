use crate::adaptive::{ActionNode, PrunedSearchPlan, RetainedSearchPlan};
use crate::adaptive::{ResourcePrices, SearchDecision, SearchPruneReason};

pub(super) fn choose(nodes: &[ActionNode]) -> SearchDecision {
    let action = select(nodes);
    let chosen_plan = action.as_ref().map(plan);
    SearchDecision {
        committed_actions: u8::from(action.is_some()),
        action,
        chosen_plan: chosen_plan.clone(),
        used_greedy_fallback: true,
        retained_plans: chosen_plan.into_iter().collect(),
        pruned_plans: vec![PrunedSearchPlan {
            action_ids: Vec::new(),
            reason: SearchPruneReason::ReserveUnderflow,
        }],
        pruned_plan_events_total: 1,
        pruned_plan_sample_truncated: false,
    }
}

fn select(nodes: &[ActionNode]) -> Option<ActionNode> {
    nodes
        .iter()
        .min_by_key(|node| {
            (
                node.forecast.completion.p99_ms,
                std::cmp::Reverse(node.forecast.success_bps),
                node.id,
            )
        })
        .cloned()
}

fn plan(node: &ActionNode) -> RetainedSearchPlan {
    RetainedSearchPlan {
        action_ids: vec![node.id],
        score_micros: node.value.total(node.resources, ResourcePrices::default()),
    }
}
