use crate::tests::adaptive::warp_tests::generation::generated_actions;
use crate::ActionId;

#[test]
fn abort_control_reports_its_action_for_reconciliation() {
    let generated = generated_actions(Some(200_000));

    assert!(generated
        .aborted_action_ids()
        .any(|action| action == ActionId::new(17)));
}
