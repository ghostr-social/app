use crate::manager::hedge_tail::{HedgeTailTimers, HedgeTailWake};
use crate::manager::transfers::InternalEvent;
use core::time::Duration;
use ghostr_engine::ActionId;

#[tokio::test(start_paused = true)]
async fn hedge_tail_wakes_at_the_exact_action_p95_deadline() {
    let (events, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut timers = HedgeTailTimers::default();
    let wake = HedgeTailWake::new(ActionId::new(7), 110);

    timers.reconcile(&[wake], 100, &events);
    tokio::time::advance(Duration::from_millis(9)).await;
    assert!(receiver.try_recv().is_err());

    tokio::time::advance(Duration::from_millis(1)).await;
    let InternalEvent::HedgeTail(reached) = receiver.recv().await.expect("valid test fixture")
    else {
        panic!("expected hedge-tail wake")
    };
    assert_eq!(reached, wake);
    assert!(timers.finish(reached));
}
