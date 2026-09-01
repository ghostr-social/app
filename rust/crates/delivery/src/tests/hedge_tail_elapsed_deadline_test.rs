use crate::manager::hedge_tail::{HedgeTailTimers, HedgeTailWake};
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::InternalEvent;
use core::time::Duration;
use ghostr_engine::ActionId;

#[tokio::test]
async fn an_elapsed_hedge_deadline_wakes_immediately_at_scheduling_time() {
    let (events, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut timers = HedgeTailTimers::default();
    let wake = HedgeTailWake::new(ActionId::new(11), unix_time_ms().saturating_sub(1));
    timers.reconcile_now(&[wake], &events);

    let event = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("elapsed hedge deadline must not reuse planning latency")
        .expect("timer event");
    let InternalEvent::HedgeTail(reached) = event else {
        panic!("hedge-tail event")
    };
    assert_eq!(reached, wake);
}
