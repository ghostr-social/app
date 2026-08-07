use super::scheduler_support::{context, next_outcome, next_started, no_start, note_at, request};
use super::scripted_scheduler_support::scripted_scheduler;
use crate::discovery::discovery_scheduler::RetrievalOutcome;
use crate::discovery::scheduler_feeds::FEED_REFRESH_BACKOFF;

#[tokio::test(start_paused = true)]
async fn exhausted_query_rechecks_head_after_native_backoff() {
    let mut harness = scripted_scheduler(vec![Vec::new(), vec![note_at(80)]]);
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    let feed = context("search");

    harness.handle.open_feed(feed.clone(), query);
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    no_start(&mut harness.started).await;

    tokio::time::advance(FEED_REFRESH_BACKOFF).await;
    let refreshed = next_started(&mut harness.started).await;
    assert_eq!(refreshed.plan.queries[0].filter.until, None);
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Started { .. }
    ));
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed { result: Ok(_), .. }
    ));
    harness.handle.close_feed(feed);
}

#[tokio::test(start_paused = true)]
async fn three_history_pages_are_followed_by_a_head_refresh() {
    let pages = [100, 90, 80, 70, 60]
        .map(|created_at| vec![note_at(created_at)])
        .to_vec();
    let mut harness = scripted_scheduler(pages);
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    harness.handle.open_feed(context("search"), query);

    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    for _ in 0..3 {
        next_started(&mut harness.started).await;
        next_outcome(&mut harness.outcomes).await;
        next_outcome(&mut harness.outcomes).await;
    }
    no_start(&mut harness.started).await;

    tokio::time::advance(FEED_REFRESH_BACKOFF).await;
    let refreshed = next_started(&mut harness.started).await;
    assert_eq!(refreshed.plan.queries[0].filter.until, None);
}
