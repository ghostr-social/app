use super::fresh_search_support::{planned, record};
use crate::adaptive::{DecisionReplayStatus, RetainedSearchPlan};

#[test]
fn a_coherent_forged_score_fails_the_real_search_rerun() {
    let (state, mut decision) = planned();
    let chosen = decision
        .search
        .chosen_plan
        .as_mut()
        .expect("fixture requires a selected search plan");
    chosen.score_micros = chosen.score_micros.saturating_add(1);
    let forged = chosen.clone();
    replace_retained_score(&mut decision.search.retained_plans, &forged);
    let captured = record(&state, &decision);

    assert_eq!(captured.integrity_status(), DecisionReplayStatus::Verified);
    assert_eq!(
        captured.replay_warp_search(),
        Err(DecisionReplayStatus::PlanMismatch)
    );
}

fn replace_retained_score(plans: &mut [RetainedSearchPlan], forged: &RetainedSearchPlan) {
    let matching = plans
        .iter_mut()
        .find(|plan| plan.action_ids == forged.action_ids)
        .expect("chosen plan remains in retained audit");
    matching.score_micros = forged.score_micros;
}
