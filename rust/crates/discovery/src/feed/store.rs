//! Open feeds and their assembled post lists (plan §5.3). Pages arrive
//! already parsed; the store canonicalizes, filters through the feed's
//! spec, windows pagination, and notifies subscribers when the visible
//! list changes. A fresh load rebases one second below the oldest
//! visible post; older pages advance by what was fetched.

use std::collections::HashMap;

use nostr_sdk::Timestamp;
use tokio::sync::watch;

use crate::content::deletions::DeletionIndex;
use crate::content::parsing::ParsedVideoPost;
use crate::feed::spec::FeedSpec;

mod occurrences;
mod pages;
mod progress;

/// How many rows a canonical feed keeps.
pub(crate) const FEED_POST_RETENTION: usize = 500;
/// Search and hashtag feeds keep a deeper but still bounded session window.
pub(crate) const QUERY_POST_RETENTION: usize = FEED_POST_RETENTION * 4;

/// Handle of one open feed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FeedId(pub u64);

#[derive(Debug)]
struct OpenFeed {
    spec: FeedSpec,
    posts: Vec<ParsedVideoPost>,
    occurrences: Vec<ParsedVideoPost>,
    deletions: DeletionIndex,
    cursor: Option<Timestamp>,
    in_flight: bool,
    revision: watch::Sender<u64>,
}

/// Every open feed, keyed by the id handed out at open time.
#[derive(Debug, Default)]
pub struct FeedStore {
    feeds: HashMap<FeedId, OpenFeed>,
    next_id: u64,
}

impl FeedStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_feed(&mut self, spec: FeedSpec) -> FeedId {
        self.next_id += 1;
        let id = FeedId(self.next_id);
        self.feeds.insert(id, OpenFeed::new(spec));
        id
    }

    pub fn close_feed(&mut self, feed: FeedId) {
        self.feeds.remove(&feed);
    }

    /// Ends every feed while preserving the id sequence. Receivers of
    /// the dropped revision senders close, and stale ids cannot alias a
    /// feed opened by the next account session.
    pub fn reset_session(&mut self) {
        self.feeds.clear();
    }

    pub fn spec(&self, feed: FeedId) -> &FeedSpec {
        &self.feeds[&feed].spec
    }

    /// The feed's visible posts, newest first; empty for unknown feeds.
    pub fn posts(&self, feed: FeedId) -> &[ParsedVideoPost] {
        self.feeds
            .get(&feed)
            .map(|open| open.posts.as_slice())
            .unwrap_or(&[])
    }

    /// A revision watch that ticks whenever the visible list changes.
    pub fn subscribe(&self, feed: FeedId) -> Option<watch::Receiver<u64>> {
        self.feeds.get(&feed).map(|open| open.revision.subscribe())
    }

    pub fn begin_load_more_at(
        &mut self,
        feed: FeedId,
        explicit: Option<Timestamp>,
    ) -> Option<Timestamp> {
        let open = self.feeds.get_mut(&feed)?;
        if open.in_flight {
            return None;
        }
        let cursor = explicit.or(open.cursor)?;
        open.in_flight = true;
        Some(cursor)
    }

    /// Publishes a revision without changing the list: the API layer
    /// calls it when the rest of the snapshot moved on — a page that
    /// finished or failed without adding rows still ends the wait of a
    /// pull-shaped reader.
    pub fn touch(&self, feed: FeedId) {
        if let Some(open) = self.feeds.get(&feed) {
            open.notify();
        }
    }
}

impl OpenFeed {
    fn new(spec: FeedSpec) -> Self {
        Self {
            spec,
            posts: Vec::new(),
            occurrences: Vec::new(),
            deletions: DeletionIndex::default(),
            cursor: None,
            in_flight: false,
            revision: watch::channel(0).0,
        }
    }

    fn notify(&self) {
        self.revision.send_modify(|revision| *revision += 1);
    }

    /// Applies the feed's declared visible window without moving its cursor.
    /// Cursors are computed before this runs, so trimming never rewinds them.
    fn trim(&mut self) {
        let retention = if self.spec.is_query() {
            QUERY_POST_RETENTION
        } else {
            FEED_POST_RETENTION
        };
        self.posts.truncate(retention);
    }
}
