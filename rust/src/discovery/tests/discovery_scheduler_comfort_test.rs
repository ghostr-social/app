//! A comfort transition never issues speculative queries: the radio
//! stays quiet even when an older page is known (plan §5.4).

use super::scheduler_support::{
    context, next_outcome, next_started, no_start, note_at, request, start_scheduler,
};
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn comfort_issues_no_speculative_queries() {
    let mut harness =
        start_scheduler(DataUsageLevel::Conservative, vec![note_at(100), note_at(90)]);
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;

    harness.modes.send(Mode::Comfort).expect("scheduler subscribed");

    no_start(&mut harness.started).await;
}
