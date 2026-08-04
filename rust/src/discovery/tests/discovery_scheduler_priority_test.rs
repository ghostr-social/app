//! With every slot busy, an interactive open-feed load overtakes
//! earlier-queued background work — parity: priority classes in
//! lib/core/work/retrieval_scheduler.dart.

use super::scheduler_support::{context, next_started, request, start_scheduler};
use crate::engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn open_feed_overtakes_queued_background_work() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.background(context("block1"), request());
    harness.handle.background(context("block2"), request());
    next_started(&mut harness.started).await;
    next_started(&mut harness.started).await;

    harness.handle.background(context("late"), request());
    harness.handle.open_feed(context("urgent"), request());

    harness.gate.add_permits(1);
    assert_eq!(next_started(&mut harness.started).await.context, context("urgent"));
    harness.gate.add_permits(1);
    assert_eq!(next_started(&mut harness.started).await.context, context("late"));
}
