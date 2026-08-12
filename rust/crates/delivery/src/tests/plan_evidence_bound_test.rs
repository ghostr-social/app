use crate::delivery_events::command_channel;
use ghostr_engine::adaptive::AllocationPlan;

#[test]
fn plan_evidence_retains_a_bounded_full_browser_journey() {
    let (handle, mut commands) = command_channel();
    for observed_at_ms in 0..520 {
        commands.publish_plan(observed_at_ms, AllocationPlan::default());
    }

    let history = handle.plan_history();
    assert_eq!(history.len(), 512);
    assert_eq!(history[0].revision, 9);
    assert_eq!(history[511].revision, 520);
}
