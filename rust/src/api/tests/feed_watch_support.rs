//! Shared plumbing for the feed-watcher tests: a channel-backed
//! [`FeedOut`] and a bounded receive so a missing update fails the test
//! instead of hanging it.

use crate::api::feed_types::FfiFeedUpdate;
use crate::api::feed_updates_stream::FeedOut;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

pub(crate) struct ChannelOut(pub(crate) mpsc::UnboundedSender<FfiFeedUpdate>);

impl FeedOut for ChannelOut {
    fn send(&self, update: FfiFeedUpdate) -> bool {
        self.0.send(update).is_ok()
    }
}

pub(crate) async fn next(
    updates: &mut mpsc::UnboundedReceiver<FfiFeedUpdate>,
) -> FfiFeedUpdate {
    timeout(Duration::from_secs(5), updates.recv())
        .await
        .expect("an update should arrive")
        .expect("the watcher should stay alive")
}
