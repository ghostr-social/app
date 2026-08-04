//! Focusing a context pulls its queued work ahead of everything else,
//! even across priority classes — parity: `focus` reordering in
//! lib/core/work/retrieval_scheduler.dart.

use super::scheduler_support::{context, next_started, request, start_scheduler};
use crate::engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn focused_context_overtakes_queued_interactive_work() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.background(context("block1"), request());
    harness.handle.background(context("block2"), request());
    next_started(&mut harness.started).await;
    next_started(&mut harness.started).await;

    harness.handle.open_feed(context("feed"), request());
    harness.handle.background(context("discover"), request());
    harness.handle.focus(context("discover"));

    harness.gate.add_permits(1);
    assert_eq!(next_started(&mut harness.started).await.context, context("discover"));
    harness.gate.add_permits(1);
    assert_eq!(next_started(&mut harness.started).await.context, context("feed"));
}
