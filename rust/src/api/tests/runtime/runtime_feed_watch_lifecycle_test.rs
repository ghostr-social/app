//! Feed watch inputs exist exactly while their runtime feed is open.

use crate::api::runtime::discovery::lock;
use crate::api::tests::runtime_fixture::runtime;
use crate::discovery::feed::spec::FeedSpec;
use std::sync::Arc;

#[tokio::test]
async fn closing_a_feed_ends_its_watch_and_rejects_new_watchers() {
    let runtime = runtime().await;
    let (feed, _) = lock(&runtime.state).open(FeedSpec::MainFeed { viewer: None });
    let (watched_state, mut revisions) = runtime.watch_inputs(feed).expect("open feed watch");
    assert!(Arc::ptr_eq(&watched_state, &runtime.state));

    runtime.close_feed(feed);

    assert!(revisions.changed().await.is_err());
    assert!(runtime.watch_inputs(feed).is_err());
}
