//! Runtime pagination dispatches both first-page retries and older pages.

use crate::api::feed_runtime::lock;
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::runtime_fixture::runtime;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::plan_executor::PlanFailure;
use nostr_sdk::Keys;

#[tokio::test]
async fn a_failed_first_page_is_reopened_by_runtime_pagination() {
    let runtime = runtime().await;
    let (feed, open) = lock(&runtime.state).open(FeedSpec::MainFeed { viewer: None });
    let context = open.expect("first page").context;
    lock(&runtime.state).apply(&context, Err(PlanFailure::new("relay down")));

    assert!(runtime.load_more(feed, None));
}

#[tokio::test]
async fn a_landed_first_page_dispatches_an_older_page() {
    let runtime = runtime().await;
    let keys = Keys::generate();
    let (feed, open) = lock(&runtime.state).open(FeedSpec::MainFeed { viewer: None });
    let context = open.expect("first page").context;
    lock(&runtime.state).apply(&context, Ok(vec![video_note(&keys, "clip", 40)]));

    assert!(runtime.load_more(feed, None));
}
