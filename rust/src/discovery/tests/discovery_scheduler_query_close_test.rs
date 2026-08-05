use super::scheduler_support::{context, next_outcome, next_started, no_start, request};
use super::scripted_scheduler_support::scripted_scheduler;
use crate::discovery::scheduler_feeds::QUERY_HUNT_BACKOFF;

#[tokio::test(start_paused = true)]
async fn closing_query_cancels_its_scheduled_native_hunt() {
    let mut harness = scripted_scheduler(vec![Vec::new()]);
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    let feed = context("search");

    harness.handle.open_feed(feed.clone(), query);
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    harness.handle.close_feed(feed);

    tokio::time::advance(QUERY_HUNT_BACKOFF).await;
    no_start(&mut harness.started).await;
}
