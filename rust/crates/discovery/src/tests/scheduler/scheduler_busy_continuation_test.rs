use crate::query::video_filters::DiscoveryFlow;
use crate::retrieval_types::RetrievalOutcome;
use crate::scheduler::feeds::FEED_REFRESH_BACKOFF;
use crate::tests::scheduler_support::{
    context, next_outcome, next_started, no_start, request, start_scheduler,
};
use ghostr_engine::DataUsageLevel;

#[tokio::test(start_paused = true)]
async fn delayed_head_continuation_does_not_overlap_active_context_work() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    let feed = context("search");
    let mut query = request();
    query.flow = DiscoveryFlow::Continuous;
    query.search_query = Some("ghost".to_owned());
    harness.handle.open_feed(feed.clone(), query);
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;

    harness.handle.background(feed.clone(), request());
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Started { .. }
    ));
    tokio::time::advance(FEED_REFRESH_BACKOFF).await;

    no_start(&mut harness.started).await;
    harness.gate.add_permits(1);
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { .. }
    ));
    no_start(&mut harness.started).await;
    assert!(harness.outcomes.try_recv().is_err());
    harness.handle.close_feed(feed);
}
