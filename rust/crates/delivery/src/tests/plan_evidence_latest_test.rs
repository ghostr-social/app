use crate::delivery_events::command_channel;
use ghostr_engine::adaptive::AllocationPlan;
use ghostr_engine::PostId;

#[tokio::test]
async fn focused_plan_is_published_atomically_and_wakes_observers() {
    let (handle, mut commands) = command_channel();
    let notifier = handle.plan_notifier();
    let changed = notifier.notified();
    commands.publish_focused_plan(42, Some(PostId::new("current")), AllocationPlan::default());

    changed.await;
    let current = handle.latest_plan().expect("latest plan");
    assert_eq!(current.revision, 1);
    assert_eq!(current.current, Some(PostId::new("current")));
    assert_eq!(current.observed_at_ms, 42);
}
