use super::protected_progress_ids;
use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes, ReserveConstraint,
};
use crate::{ByteRange, PostId};

#[test]
fn chance_qualified_path_wins_over_faster_unprotected_progress() {
    let fast_unprotected = node(1, 1, 1_000);
    let qualified = node(2, 1_000, 10_000);
    let reserve = ReserveConstraint {
        protected_action_ids: vec![qualified.id],
        ..ReserveConstraint::default()
    };
    let preferred = protected_progress_ids(&reserve, vec![fast_unprotected.id, qualified.id]);

    let selected = super::least_risk::choose(&[fast_unprotected, qualified], &preferred)
        .action
        .expect("protected reserve action");

    assert_eq!(selected.id, 2);
}

fn node(id: u16, p99_ms: u64, success_bps: u16) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("reserve"),
        ActionKind::FetchRange(ByteRange::new(u64::from(id), u64::from(id) + 1)),
        ActionValue::default(),
    )
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(1, 1, p99_ms, p99_ms),
        success_bps,
        1_000,
    ))
}
