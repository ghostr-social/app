use crate::manager::reconcile_warp;
use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::StorageSnapshot;

#[test]
fn advanced_decision_executes_only_its_selected_transfer() {
    let work = plan(2_000, 2_000_000, StorageSnapshot::new(2_000_000_000, 0));
    assert!(work.warp.is_some());
    let selected: Vec<_> = work
        .selected_transfers
        .iter()
        .map(|item| item.id())
        .collect();
    let legacy: Vec<_> = work.transfers.iter().map(|item| item.id()).collect();

    let execution = reconcile_warp::execution(work);
    let actual: Vec<_> = execution.transfers.iter().map(|item| item.id()).collect();

    assert_eq!(actual, selected);
    assert_ne!(
        actual, legacy,
        "legacy parallel allocations must not execute"
    );
}
