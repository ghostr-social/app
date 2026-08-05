//! A handle whose owning runtime ended reports a stopped scheduler.

use super::scheduler_support::{context, next_outcome, next_started, request, start_scheduler};
use super::scripted_scheduler_support::scripted_scheduler;
use crate::engine::DataUsageLevel;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time::timeout;

#[tokio::test]
async fn dropping_the_last_handle_stops_the_scheduler_worker() {
    let harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    let mut outcomes = harness.outcomes;

    drop(harness.handle);

    let closed = timeout(Duration::from_millis(100), outcomes.recv())
        .await
        .expect("worker should stop");
    assert!(closed.is_none());
}

#[tokio::test]
async fn dropping_the_last_handle_aborts_an_active_retrieval() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.open_feed(context("active"), request());
    next_started(&mut harness.started).await;

    drop(harness.handle);

    let closed = timeout(Duration::from_millis(100), harness.outcomes.recv())
        .await
        .expect("active retrieval should abort");
    assert!(closed.is_none());
}

#[tokio::test]
async fn a_delayed_query_hunt_does_not_own_the_scheduler() {
    let mut harness = scripted_scheduler(vec![Vec::new()]);
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    harness.handle.open_feed(context("search"), query);
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    drop(harness.handle);
    tokio::task::yield_now().await;

    assert!(matches!(
        harness.outcomes.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)
    ));
}

#[test]
fn reset_after_scheduler_shutdown_reports_stopped() {
    let owner = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("owner runtime");
    let handle =
        owner.block_on(async { start_scheduler(DataUsageLevel::Conservative, Vec::new()).handle });
    drop(owner);
    let caller = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("caller runtime");

    let failure = caller
        .block_on(handle.reset_session())
        .expect_err("the scheduler task ended with its runtime");

    assert_eq!(failure.message, "the discovery scheduler stopped");
}
