use super::action_ids;
use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, AdaptivePlayabilityPolicy,
    CompletionTimes, RetrievalRequest,
};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn shifted_partial_overlap_is_not_ordered_reserve_progress() {
    let mut state = snapshot(6, 2_500_000, 20_000, 120);
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    for candidate in &mut state.candidates[1..=4] {
        candidate.present = candidate.startup.as_ref().unwrap().ranges().to_vec();
    }
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let work = base
        .allocations
        .iter()
        .find(|item| item.post == state.candidates[5].post)
        .expect("fifth reserve work");
    let planned = work.request.requested_bytes();
    let partial = ByteRange::new(planned.start + 1, planned.end);
    let node = ActionNode::new(
        1,
        work.post.clone(),
        ActionKind::FetchRange(partial),
        ActionValue::default(),
    )
    .with_request(RetrievalRequest::FetchRange {
        bytes: partial,
        promotion: None,
    })
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(1, 1, 1, 1),
        10_000,
        1,
    ));

    assert!(action_ids(&state, &base, &[node]).is_empty());
}
