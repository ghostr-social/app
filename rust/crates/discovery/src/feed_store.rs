//! Open feeds and their assembled post lists (plan §5.3). Pages arrive
//! already parsed; the store canonicalizes, filters through the feed's
//! spec, windows pagination, and notifies subscribers when the visible
//! list changes. A fresh load rebases one second below the oldest
//! visible post; older pages advance by what was fetched.

use std::collections::HashMap;

use nostr_sdk::Timestamp;
use tokio::sync::watch;

use crate::event_parsing::ParsedVideoPost;
use crate::feed_assembly::{append_new, select_posts};
use crate::feed_spec::FeedSpec;
use crate::feed_store_cursor::{older_cursor, post_cursor};
use crate::social_graph::SocialGraph;

mod progress;

/// How many rows a canonical feed keeps. Query feeds preserve their complete
/// discovered history so native snapshots can expose result 501 and beyond.
pub const FEED_POST_RETENTION: usize = 500;

/// Handle of one open feed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FeedId(pub u64);

#[derive(Debug)]
struct OpenFeed {
    spec: FeedSpec,
    posts: Vec<ParsedVideoPost>,
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

    /// Replaces the feed with a fresh first page and restarts pagination.
    pub fn ingest_first_page(
        &mut self,
        feed: FeedId,
        fetched: Vec<ParsedVideoPost>,
        graph: &SocialGraph,
    ) {
        let Some(open) = self.feeds.get_mut(&feed) else {
            return;
        };
        open.posts = select_posts(&open.spec, fetched, graph);
        open.cursor = post_cursor(&open.posts);
        open.in_flight = false;
        open.trim();
        open.notify();
    }

    /// Claims the cursor for one older request; `None` when the feed is
    /// exhausted or a request is already in flight (`beginLoad`).
    pub fn begin_load_more(&mut self, feed: FeedId) -> Option<Timestamp> {
        self.begin_load_more_at(feed, None)
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

    /// Releases the in-flight claim after a failed older request
    /// (`failLoad`); the cursor stays for the next swipe.
    pub fn fail_load_more(&mut self, feed: FeedId) {
        if let Some(open) = self.feeds.get_mut(&feed) {
            open.in_flight = false;
        }
    }

    /// Advances pagination from raw wire events, including matching notes
    /// that were not playable enough to become rows.
    pub fn set_retrieval_cursor(&mut self, feed: FeedId, cursor: Option<Timestamp>) {
        if let (Some(open), Some(cursor)) = (self.feeds.get_mut(&feed), cursor) {
            open.cursor = Some(cursor);
        }
    }

    /// Appends one fetched older page and advances the cursor by what was
    /// fetched, so pages full of filtered creators cannot stall
    /// pagination; subscribers hear only when something was appended.
    /// Reports whether they did.
    pub fn ingest_older_page(
        &mut self,
        feed: FeedId,
        fetched: Vec<ParsedVideoPost>,
        graph: &SocialGraph,
    ) -> bool {
        let Some(open) = self.feeds.get_mut(&feed) else {
            return false;
        };
        open.in_flight = false;
        open.cursor = older_cursor(&open.spec, open.cursor, &fetched);
        let page = select_posts(&open.spec, fetched, graph);
        if !append_new(&mut open.posts, page) {
            return false;
        }
        open.trim();
        open.notify();
        true
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
            cursor: None,
            in_flight: false,
            revision: watch::channel(0).0,
        }
    }

    fn notify(&self) {
        self.revision.send_modify(|revision| *revision += 1);
    }

    /// Bounds canonical feeds while query feeds preserve discovered history.
    /// Cursors are computed before this runs, so trimming never rewinds them.
    fn trim(&mut self) {
        if !self.spec.is_query() {
            self.posts.truncate(FEED_POST_RETENTION);
        }
    }
}
