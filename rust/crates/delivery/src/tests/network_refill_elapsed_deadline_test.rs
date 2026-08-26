use crate::manager::network_refill_timer::NetworkRefillTimer;
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::InternalEvent;
use core::time::Duration;

#[tokio::test]
async fn an_elapsed_deadline_wakes_immediately_at_scheduling_time() {
    let (events, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut timer = NetworkRefillTimer::default();
    timer.reconcile_now(Some(unix_time_ms().saturating_sub(1)), &events);

    let event = tokio::time::timeout(Duration::from_millis(50), receiver.recv())
        .await
        .expect("elapsed deadline must not reuse planning latency")
        .expect("timer event");
    assert!(matches!(event, InternalEvent::NetworkRefill(_)));
}
