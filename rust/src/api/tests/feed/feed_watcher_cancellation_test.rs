//! A cancelled feed receiver ends its watcher after the baseline snapshot.

use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedUpdate;
use crate::api::feed_updates_stream::{watch_feed, FeedOut};
use crate::api::runtime::discovery::{lock, SharedFeedState};
use crate::discovery::feed::spec::FeedSpec;
use std::sync::{Arc, Mutex};

struct RejectingOut(Arc<Mutex<Option<FfiFeedUpdate>>>);

impl FeedOut for RejectingOut {
    fn send(&self, update: FfiFeedUpdate) -> bool {
        *self.0.lock().expect("update capture") = Some(update);
        false
    }
}

#[tokio::test]
async fn a_closed_receiver_gets_one_baseline_before_the_watcher_ends() {
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    let (feed, _) = lock(&state).open(FeedSpec::MainFeed { viewer: None });
    let revisions = lock(&state).subscribe(feed).expect("open feed");
    let captured = Arc::new(Mutex::new(None));

    watch_feed(RejectingOut(captured.clone()), state, feed, revisions).await;

    let update = captured.lock().expect("update capture").clone();
    assert_eq!(update.expect("baseline").feed_id, feed.0.to_string());
}
