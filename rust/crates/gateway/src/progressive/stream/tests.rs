use super::{read_with_armed_change, wait_for_store_change, PumpStep};
use std::future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

#[tokio::test]
async fn store_change_is_armed_before_the_range_read() {
    let notify = Arc::new(Notify::new());
    let during_read = notify.clone();
    let ((), changed) = read_with_armed_change(&notify, async move {
        during_read.notify_waiters();
    })
    .await;

    tokio::time::timeout(Duration::from_millis(10), changed)
        .await
        .expect("change emitted during the read must remain observable");
}

#[tokio::test(start_paused = true)]
async fn idle_deadline_wins_over_an_unrelated_store_change() {
    let result = wait_for_store_change(
        tokio::time::Instant::now(),
        future::ready(()),
        future::pending(),
    )
    .await;

    assert!(matches!(result, PumpStep::TimedOut));
}
