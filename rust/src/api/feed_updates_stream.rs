//! R→D feed updates (plan §2 phase-2 additions). Each subscription
//! runs its own watcher on the feed's revision watch: it first sends a
//! baseline snapshot, then one full snapshot per visible-list change.
//! The stream ends when the feed closes or Dart cancels it.

use crate::api::feed::mapping::parse_feed_id;
use crate::api::feed::projection::project;
use crate::api::feed_types::FfiFeedUpdate;
use crate::api::runtime::discovery::{lock, SharedFeedState};
use crate::api::runtime::registry;
use crate::discovery::feed::store::FeedId;
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use tokio::sync::watch;

/// Where feed updates go; lets tests observe without a Dart sink.
pub(crate) trait FeedOut: Send + 'static {
    /// Returns `false` once the receiver is gone: the watcher stops.
    fn send(&self, update: FfiFeedUpdate) -> bool;
}

impl FeedOut for StreamSink<FfiFeedUpdate> {
    fn send(&self, update: FfiFeedUpdate) -> bool {
        self.add(update).is_ok()
    }
}

/// Subscribes to one open feed's snapshots.
#[frb]
pub async fn ffi_feed_updates(
    feed_id: String,
    sink: StreamSink<FfiFeedUpdate>,
) -> anyhow::Result<()> {
    let feed = parse_feed_id(&feed_id)?;
    let engine = registry::engine()?;
    let (state, revisions) = engine.discovery.watch_inputs(feed)?;
    tokio::spawn(watch_feed(sink, state, feed, revisions));
    Ok(())
}

pub(crate) async fn watch_feed(
    out: impl FeedOut,
    state: SharedFeedState,
    feed: FeedId,
    mut revisions: watch::Receiver<u64>,
) {
    loop {
        let revision = *revisions.borrow_and_update();
        if !out.send(snapshot_update(&state, feed, revision)) {
            return;
        }
        if revisions.changed().await.is_err() {
            return;
        }
    }
}

/// Stage and rows are read under one lock so a snapshot can never
/// claim to be settled while showing the previous page's rows.
fn snapshot_update(state: &SharedFeedState, feed: FeedId, revision: u64) -> FfiFeedUpdate {
    let state = lock(state);
    let projection = project(&state, feed);
    FfiFeedUpdate {
        feed_id: feed.0.to_string(),
        revision,
        stage: projection.stage,
        posts: projection.posts,
    }
}
