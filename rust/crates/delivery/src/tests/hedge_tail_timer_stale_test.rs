use crate::manager::hedge_tail::{HedgeTailTimers, HedgeTailWake};
use crate::manager::transfers::InternalEvent;
use ghostr_engine::ActionId;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn a_replaced_action_deadline_cannot_emit_a_stale_tail_wake() {
    let (events, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut timers = HedgeTailTimers::default();
    let old = HedgeTailWake::new(ActionId::new(9), 110);
    let current = HedgeTailWake::new(ActionId::new(9), 120);

    timers.reconcile(&[old], 100, &events);
    timers.reconcile(&[current], 100, &events);
    tokio::time::advance(Duration::from_millis(10)).await;
    assert!(receiver.try_recv().is_err());

    tokio::time::advance(Duration::from_millis(10)).await;
    let InternalEvent::HedgeTail(reached) = receiver.recv().await.unwrap() else {
        panic!("expected hedge-tail wake")
    };
    assert_eq!(reached, current);
    assert!(!timers.finish(old));
    assert!(timers.finish(reached));
}
