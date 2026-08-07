//! The worker pool bounds concurrent retrievals at the data-usage cap
//! (conservative: 2), admitting queued work only as slots free up.

use crate::tests::scheduler_support::{context, next_started, no_start, request, start_scheduler};
use ghostr_engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn conservative_level_runs_two_retrievals_at_once() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());

    for name in ["a", "b", "c"] {
        harness.handle.background(context(name), request());
    }

    assert_eq!(
        next_started(&mut harness.started).await.context,
        context("a")
    );
    assert_eq!(
        next_started(&mut harness.started).await.context,
        context("b")
    );
    no_start(&mut harness.started).await;

    harness.gate.add_permits(1);
    assert_eq!(
        next_started(&mut harness.started).await.context,
        context("c")
    );
}
