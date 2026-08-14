use crate::query::video_filters::DiscoveryFlow;
use crate::retrieval_types::{RetrievalOutcome, RetrievalPriority};
use crate::tests::scheduler_support::{
    context, next_outcome, next_started, no_start, note_at, request, start_scheduler,
};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Timestamp;

#[tokio::test(start_paused = true)]
async fn one_feed_never_runs_overlapping_retrievals() {
    let mut harness = start_scheduler(
        DataUsageLevel::Conservative,
        vec![note_at(100), note_at(90)],
    );
    let feed = context("feed");
    let mut discovery = request();
    discovery.flow = DiscoveryFlow::Continuous;
    harness.handle.open_feed(feed.clone(), discovery);
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;
    next_started(&mut harness.started).await;
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Started { .. }
    ));

    harness
        .handle
        .load_more(feed.clone(), Some(Timestamp::from(50)));
    no_start(&mut harness.started).await;
    harness.gate.add_permits(1);
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { .. }
    ));
    let older = next_started(&mut harness.started).await;

    assert_eq!(older.context, feed);
    assert_eq!(older.priority, RetrievalPriority::Interactive);
    assert_eq!(
        older.plan.queries[0].filter.until,
        Some(Timestamp::from(50))
    );
    assert!(harness.outcomes.try_recv().is_err());
}
