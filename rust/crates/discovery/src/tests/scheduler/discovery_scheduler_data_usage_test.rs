//! The data-usage knob sets the worker-pool cap (2/4/6) and applies
//! live — parity: `maxConcurrentRequests` in
//! lib/features/settings/domain/data_usage_level.dart.

use crate::scheduler::max_concurrent_requests;
use crate::tests::scheduler_support::{context, next_started, no_start, request, start_scheduler};
use ghostr_engine::DataUsageLevel;

#[test]
fn caps_mirror_dart_data_usage_levels() {
    assert_eq!(max_concurrent_requests(DataUsageLevel::Conservative), 2);
    assert_eq!(max_concurrent_requests(DataUsageLevel::Balanced), 4);
    assert_eq!(max_concurrent_requests(DataUsageLevel::Aggressive), 6);
}

#[tokio::test(start_paused = true)]
async fn raising_the_level_admits_more_retrievals() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    for name in ["a", "b", "c", "d", "e", "f", "g"] {
        harness.handle.background(context(name), request());
    }
    next_started(&mut harness.started).await;
    next_started(&mut harness.started).await;
    no_start(&mut harness.started).await;

    harness.handle.set_data_usage(DataUsageLevel::Aggressive);

    for _ in 0..4 {
        next_started(&mut harness.started).await;
    }
    no_start(&mut harness.started).await;
}
