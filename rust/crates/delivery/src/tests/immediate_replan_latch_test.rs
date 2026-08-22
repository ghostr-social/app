use crate::manager::immediate_replan::ImmediateReplan;
use crate::manager::transfers::InternalEvent;
use tokio::sync::mpsc;

#[test]
fn immediate_replans_coalesce_until_the_queued_wake_is_consumed() {
    let (events, mut receiver) = mpsc::unbounded_channel();
    let mut replans = ImmediateReplan::default();

    assert!(replans.request(&events));
    assert!(!replans.request(&events));
    assert!(matches!(
        receiver.try_recv(),
        Ok(InternalEvent::ImmediateReplan)
    ));
    assert!(receiver.try_recv().is_err());

    replans.consume();
    assert!(replans.request(&events));
    assert!(matches!(
        receiver.try_recv(),
        Ok(InternalEvent::ImmediateReplan)
    ));
}

#[test]
fn failed_immediate_replan_send_releases_the_latch() {
    let (closed, receiver) = mpsc::unbounded_channel();
    drop(receiver);
    let mut replans = ImmediateReplan::default();

    assert!(!replans.request(&closed));

    let (events, mut receiver) = mpsc::unbounded_channel();
    assert!(replans.request(&events));
    assert!(matches!(
        receiver.try_recv(),
        Ok(InternalEvent::ImmediateReplan)
    ));
}
