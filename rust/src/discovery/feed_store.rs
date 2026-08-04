//! Open feeds and their assembled post lists (plan §5.3). Pages arrive
//! already parsed; the store canonicalizes, filters through the feed's
//! spec, windows pagination, and notifies subscribers when the visible
//! list changes. Cursor semantics mirror `FeedPagination`
//! (lib/features/video_catalog/presentation/feed_pagination.dart): a
//! fresh load rebases one second below the oldest *visible* post
//! (feed_cubit.dart `_acceptLoad`), older pages advance by what was
//! *fetched* (filtered_video_feed_repository.dart `_nextCursor`).

use std::collections::HashMap;

use nostr_sdk::Timestamp;
use tokio::sync::watch;

use crate::discovery::event_parsing::ParsedVideoPost;
use crate::discovery::feed_assembly::{append_new, select_posts};
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::pagination::next_page_cursor;
use crate::discovery::social_graph::SocialGraph;

/// How many rows one open feed keeps — about ten pages. Feeds now stay
/// open for the life of the Dart source
/// (lib/features/video_catalog/data/rust_feed_sessions.dart) while the
/// scheduler prefetches older pages into the active one, so an
/// unbounded list would grow all session, and every revision ships the
/// whole list over the FFI. The window is anchored at the head: the
/// newest rows are what a returning pull answers with. Past it the
/// feed still paginates on the relay side (cursors follow what was
/// fetched) and Dart keeps the list a viewer scrolled through; if the
/// limit ever bites, window the snapshot around the reader's cursor
/// rather than growing this number.
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

    pub fn spec(&self, feed: FeedId) -> Option<&FeedSpec> {
        self.feeds.get(&feed).map(|open| &open.spec)
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
        open.cursor = next_page_cursor(created_at(&open.posts));
        open.in_flight = false;
        open.trim();
        open.notify();
    }

    /// Claims the cursor for one older request; `None` when the feed is
    /// exhausted or a request is already in flight (`beginLoad`).
    pub fn begin_load_more(&mut self, feed: FeedId) -> Option<Timestamp> {
        let open = self.feeds.get_mut(&feed)?;
        if open.in_flight {
            return None;
        }
        let cursor = open.cursor?;
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

    /// Drops the rows past the retention window. Cursors are computed
    /// from what was fetched before this runs, so a trim never rewinds
    /// pagination.
    fn trim(&mut self) {
        self.posts.truncate(FEED_POST_RETENTION);
    }
}

/// An empty page exhausts a canonical feed but leaves a query feed its
/// cursor — the search keeps hunting (query_video_feed_repository.dart).
fn older_cursor(
    spec: &FeedSpec,
    current: Option<Timestamp>,
    fetched: &[ParsedVideoPost],
) -> Option<Timestamp> {
    match next_page_cursor(created_at(fetched)) {
        Some(next) => Some(next),
        None if spec.exhausts_on_empty_page() => None,
        None => current,
    }
}

fn created_at(posts: &[ParsedVideoPost]) -> impl Iterator<Item = Timestamp> + '_ {
    posts.iter().map(|post| Timestamp::from(post.created_at))
}
