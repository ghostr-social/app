//! Reset aborts old retrievals and releases every worker slot immediately.

use crate::tests::scheduler_support::{context, next_started, request, start_scheduler};
use ghostr_engine::DataUsageLevel;

#[tokio::test]
async fn fresh_work_starts_without_waiting_for_old_retrievals() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.background(context("old-a"), request());
    harness.handle.background(context("old-b"), request());
    next_started(&mut harness.started).await;
    next_started(&mut harness.started).await;

    harness
        .handle
        .reset_session()
        .await
        .expect("scheduler reset");
    harness.handle.background(context("fresh"), request());

    let started = next_started(&mut harness.started).await;
    assert_eq!(started.context, context("fresh"));
}
