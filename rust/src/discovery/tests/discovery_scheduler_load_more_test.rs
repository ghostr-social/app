//! Load-more issues an interactive older page: an explicit cursor wins
//! over the tracked one, and without any cursor nothing is issued
//! (plan §2 `ffi_load_more`).

use super::scheduler_support::{context, next_started, no_start, request, start_scheduler};
use crate::discovery::retrieval_queue::RetrievalPriority;
use crate::engine::DataUsageLevel;
use nostr_sdk::Timestamp;

#[tokio::test(start_paused = true)]
async fn explicit_cursor_drives_the_older_page() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;

    harness
        .handle
        .load_more(context("feed"), Some(Timestamp::from(50)));

    let page = next_started(&mut harness.started).await;
    assert_eq!(page.priority, RetrievalPriority::Interactive);
    assert_eq!(page.plan.queries[0].filter.until, Some(Timestamp::from(50)));
}

#[tokio::test(start_paused = true)]
async fn load_more_without_any_cursor_is_ignored() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;

    harness.handle.load_more(context("feed"), None);

    no_start(&mut harness.started).await;
}

#[tokio::test(start_paused = true)]
async fn load_more_replaces_a_query_hunt_with_the_requested_page() {
    let mut harness = start_scheduler(DataUsageLevel::Conservative, Vec::new());
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    harness.handle.open_feed(context("search"), query);
    next_started(&mut harness.started).await;

    harness
        .handle
        .load_more(context("search"), Some(Timestamp::from(50)));

    let page = next_started(&mut harness.started).await;
    assert_eq!(page.plan.queries[0].filter.until, Some(Timestamp::from(50)));
}
