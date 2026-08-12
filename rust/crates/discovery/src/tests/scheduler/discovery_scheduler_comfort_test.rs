//! Held demand never issues speculative queries: the radio stays quiet
//! even when an older page is known (plan §5.4).

use crate::tests::scheduler_support::{
    context, next_outcome, next_started, no_start, note_at, request, start_scheduler,
};
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn held_demand_issues_no_speculative_queries() {
    let mut harness = start_scheduler(
        DataUsageLevel::Conservative,
        vec![note_at(100), note_at(90)],
    );
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;

    harness
        .demand
        .send(DiscoveryDemand::Hold)
        .expect("scheduler subscribed");

    no_start(&mut harness.started).await;
}

#[tokio::test(start_paused = true)]
async fn closed_demand_channel_does_not_stop_command_processing() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    let handle = harness.handle.clone();
    drop(harness.demand);
    tokio::task::yield_now().await;

    handle.background(context("background"), request());
    next_started(&mut harness.started).await;
    tokio::task::yield_now().await;
}
