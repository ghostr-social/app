use super::scheduler_support::{context, next_outcome, next_started, no_start, note_at};
use super::scripted_scheduler_support::scripted_scheduler;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::scheduler_feeds::FEED_REFRESH_BACKOFF;
use crate::discovery::social_graph::SocialGraph;
use crate::discovery::video_filters::DiscoveryRequest;
use nostr_sdk::{Keys, Timestamp};

fn main_request() -> DiscoveryRequest {
    let graph = SocialGraph::new(Keys::generate().public_key());
    FeedSpec::MainFeed { viewer: None }
        .page_request(None, &graph)
        .expect("main feed request")
}

#[tokio::test]
async fn main_feed_ingests_older_pages_without_ui_or_inventory_commands() {
    let mut harness = scripted_scheduler(vec![vec![note_at(100)], vec![note_at(90)]]);
    let feed = context("main");

    harness.handle.open_feed(feed.clone(), main_request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;

    let older = next_started(&mut harness.started).await;
    assert_eq!(
        older.plan.queries[0].filter.until,
        Some(Timestamp::from(99))
    );
    next_outcome(&mut harness.outcomes).await;
    next_outcome(&mut harness.outcomes).await;
    harness.handle.close_feed(feed);
}

#[tokio::test(start_paused = true)]
async fn exhausted_main_history_returns_to_head_discovery() {
    let mut harness = scripted_scheduler(vec![vec![note_at(100)], Vec::new(), vec![note_at(80)]]);
    let feed = context("main");
    harness.handle.open_feed(feed.clone(), main_request());
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    next_started(&mut harness.started).await;
    next_outcome(&mut harness.outcomes).await;
    next_outcome(&mut harness.outcomes).await;
    no_start(&mut harness.started).await;

    tokio::time::advance(FEED_REFRESH_BACKOFF).await;

    let refreshed = next_started(&mut harness.started).await;
    assert_eq!(refreshed.plan.queries[0].filter.until, None);
    harness.handle.close_feed(feed);
}
