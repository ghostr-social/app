use crate::manager::network_refill_timer::NetworkRefillTimer;
use crate::manager::transfers::InternalEvent;
use core::time::Duration;

#[tokio::test(start_paused = true)]
async fn refill_wake_is_exact_replaceable_and_cancellable() {
    let (events, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut timer = NetworkRefillTimer::default();
    let stale = timer
        .reconcile(Some(110), 100, &events)
        .expect("valid test fixture");
    timer.reconcile(None, 100, &events);
    let current = timer
        .reconcile(Some(110), 100, &events)
        .expect("valid test fixture");

    assert_ne!(stale, current);
    assert!(!timer.finish(stale));
    tokio::time::advance(Duration::from_millis(10)).await;
    let InternalEvent::NetworkRefill(reached) = receiver.recv().await.expect("valid test fixture")
    else {
        panic!("expected network-refill wake")
    };
    assert_eq!(reached, current);
    assert!(timer.finish(reached));

    timer.reconcile(Some(120), 110, &events);
    timer.reconcile(None, 110, &events);
    tokio::time::advance(Duration::from_millis(10)).await;
    assert!(receiver.try_recv().is_err());
}
