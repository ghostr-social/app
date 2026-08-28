pub(super) use super::warp_planner_promotion_contract_test::support::generated_actions;
use crate::adaptive::ActionKind;

#[test]
fn adaptive_dag_generates_response_ready_promotion_and_control_actions() {
    let generated = generated_actions(Some(200_000));
    let kinds: Vec<_> = generated
        .actions
        .iter()
        .map(|item| &item.node.kind)
        .collect();
    assert!(kinds.iter().any(|kind| matches!(kind, ActionKind::Head)));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Prefix(_))));
    assert!(kinds.iter().any(|kind| matches!(kind, ActionKind::Tail(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::FetchRange(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Promote { .. })));
    assert!(kinds
        .iter()
        .all(|kind| !matches!(kind, ActionKind::Transform(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::CacheUpgrade(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Hedge { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Cancel(_))));
}
