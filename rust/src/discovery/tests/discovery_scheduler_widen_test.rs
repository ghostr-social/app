//! Hunger over an exhausted feed widens the primary query once to the
//! wide limit; a second hunger transition stays idle (plan §5.4).

use super::scheduler_support::{
    context, next_outcome, next_started, no_start, request, start_scheduler,
};
use crate::discovery::retrieval_queue::RetrievalPriority;
use crate::discovery::video_filters::WIDE_QUERY_LIMIT;
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn hunger_widens_an_exhausted_feed_once() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;

    harness
        .modes
        .send(Mode::Hunger)
        .expect("scheduler subscribed");

    let widened = next_started(&mut harness.started).await;
    assert_eq!(widened.priority, RetrievalPriority::Background);
    let primary = &widened.plan.queries[0].filter;
    assert_eq!(primary.limit, Some(WIDE_QUERY_LIMIT));
    assert_eq!(primary.until, None);

    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;
    harness
        .modes
        .send(Mode::Comfort)
        .expect("scheduler subscribed");
    harness
        .modes
        .send(Mode::Hunger)
        .expect("scheduler subscribed");
    no_start(&mut harness.started).await;
}
