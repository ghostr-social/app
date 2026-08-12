//! Expansion demand over an exhausted feed widens the primary query once;
//! a second expansion transition stays idle (plan §5.4).

use crate::query::video_filters::WIDE_QUERY_LIMIT;
use crate::retrieval_types::RetrievalPriority;
use crate::tests::scheduler_support::{
    context, next_outcome, next_started, no_start, request, start_scheduler,
};
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn expansion_widens_an_exhausted_feed_once() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;

    harness
        .demand
        .send(DiscoveryDemand::Expand)
        .expect("scheduler subscribed");

    let widened = next_started(&mut harness.started).await;
    assert_eq!(widened.priority, RetrievalPriority::Background);
    let primary = &widened.plan.queries[0].filter;
    assert_eq!(primary.limit, Some(WIDE_QUERY_LIMIT));
    assert_eq!(primary.until, None);

    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;
    harness
        .demand
        .send(DiscoveryDemand::Hold)
        .expect("scheduler subscribed");
    harness
        .demand
        .send(DiscoveryDemand::Expand)
        .expect("scheduler subscribed");
    no_start(&mut harness.started).await;
}
