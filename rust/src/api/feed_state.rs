//! API-side bookkeeping of every open feed: routes retrieval outcomes
//! into the `FeedStore`, claims load-more cursors, and builds full row
//! snapshots. Pure state — the FFI runtime wraps it in a lock and
//! forwards the returned dispatches to the discovery scheduler.

use crate::api::feed_decisions::{LoadMoreAction, LoadMoreDecision, OpenDispatch};
use crate::api::feed_mapping::{feed_post, resolved_creator};
use crate::api::feed_types::FfiFeedPost;
use crate::discovery::event_parsing::{video_post_from_event, ParsedVideoPost};
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::feed_store::{FeedId, FeedStore};
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::profile_store::ProfileStore;
use crate::discovery::retrieval_queue::FeedContext;
use crate::discovery::social_graph::SocialGraph;
use flutter_rust_bridge::frb;
use nostr_sdk::{Event, Keys, Timestamp};
use std::collections::HashMap;
use tokio::sync::watch;

#[derive(Debug)]
struct FeedProgress {
    context: FeedContext,
    first_loaded: bool,
    awaiting_first: bool,
    awaiting_more: bool,
}

/// Every open feed plus the shared profile store and social graph.
/// The graph follows the most recent main-feed viewer; until one opens
/// it belongs to a throwaway session key, so nothing is ever muted.
#[frb(ignore)]
#[derive(Debug)]
pub(crate) struct FeedState {
    store: FeedStore,
    profiles: ProfileStore,
    graph: SocialGraph,
    feeds: HashMap<FeedId, FeedProgress>,
}

impl FeedState {
    pub(crate) fn new() -> Self {
        Self {
            store: FeedStore::new(),
            profiles: ProfileStore::new(),
            graph: SocialGraph::new(Keys::generate().public_key()),
            feeds: HashMap::new(),
        }
    }

    /// Opens the feed and returns its first-page dispatch; `None` when
    /// the spec can never produce content (blank search parity).
    pub(crate) fn open(&mut self, spec: FeedSpec) -> (FeedId, Option<OpenDispatch>) {
        self.adopt_viewer(&spec);
        let feed = self.store.open_feed(spec.clone());
        let context = FeedContext::new(format!("feed-{}", feed.0));
        let dispatch = spec
            .page_request(None)
            .map(|request| OpenDispatch { context: context.clone(), request });
        let progress = FeedProgress {
            context,
            first_loaded: false,
            awaiting_first: dispatch.is_some(),
            awaiting_more: false,
        };
        self.feeds.insert(feed, progress);
        (feed, dispatch)
    }

    pub(crate) fn close(&mut self, feed: FeedId) {
        self.store.close_feed(feed);
        self.feeds.remove(&feed);
    }

    pub(crate) fn subscribe(&self, feed: FeedId) -> Option<watch::Receiver<u64>> {
        self.store.subscribe(feed)
    }

    /// Claims the next older page. One request per feed is in flight
    /// at a time; an explicit cursor wins over the claimed one
    /// (scheduler `LoadMore` parity).
    pub(crate) fn load_more(
        &mut self,
        feed: FeedId,
        explicit: Option<Timestamp>,
    ) -> LoadMoreDecision {
        let Some(progress) = self.feeds.get(&feed) else {
            return LoadMoreDecision::finished();
        };
        if progress.awaiting_first || progress.awaiting_more {
            return LoadMoreDecision::wait();
        }
        let context = progress.context.clone();
        if !progress.first_loaded {
            return self.reopen(feed, context);
        }
        self.claim_older(feed, context, explicit)
    }

    /// Routes one retrieval outcome into the store: the first landed
    /// page replaces the feed, every later one (interactive or
    /// background prefetch) appends as an older page.
    pub(crate) fn apply(&mut self, context: &FeedContext, result: Result<Vec<Event>, PlanFailure>) {
        let Some(feed) = self.feed_for(context) else {
            return;
        };
        match result {
            Ok(events) => self.ingest_page(feed, &events),
            Err(_) => self.record_failure(feed),
        }
    }

    /// The feed's visible rows, newest first, creators resolved.
    pub(crate) fn snapshot(&self, feed: FeedId) -> Vec<FfiFeedPost> {
        self.store
            .posts(feed)
            .iter()
            .map(|post| feed_post(post, resolved_creator(&self.profiles, post)))
            .collect()
    }

    fn adopt_viewer(&mut self, spec: &FeedSpec) {
        if let FeedSpec::MainFeed { viewer } = spec {
            self.graph = SocialGraph::new(*viewer);
        }
    }

    fn reopen(&mut self, feed: FeedId, context: FeedContext) -> LoadMoreDecision {
        let request = self.store.spec(feed).and_then(|spec| spec.page_request(None));
        let Some(request) = request else {
            return LoadMoreDecision::finished();
        };
        self.mark(feed, |progress| progress.awaiting_first = true);
        LoadMoreDecision {
            may_have_more: true,
            action: LoadMoreAction::Reopen(OpenDispatch { context, request }),
        }
    }

    fn claim_older(
        &mut self,
        feed: FeedId,
        context: FeedContext,
        explicit: Option<Timestamp>,
    ) -> LoadMoreDecision {
        let Some(cursor) = self.store.begin_load_more(feed) else {
            return LoadMoreDecision::finished();
        };
        self.mark(feed, |progress| progress.awaiting_more = true);
        LoadMoreDecision {
            may_have_more: true,
            action: LoadMoreAction::Older { context, older_than: explicit.unwrap_or(cursor) },
        }
    }

    fn ingest_page(&mut self, feed: FeedId, events: &[Event]) {
        for event in events {
            self.profiles.ingest(event);
        }
        let posts: Vec<ParsedVideoPost> = events.iter().filter_map(video_post_from_event).collect();
        if self.first_loaded(feed) {
            self.store.ingest_older_page(feed, posts, &self.graph);
        } else {
            self.store.ingest_first_page(feed, posts, &self.graph);
        }
        self.mark(feed, |progress| {
            progress.first_loaded = true;
            progress.awaiting_first = false;
            progress.awaiting_more = false;
        });
    }

    fn record_failure(&mut self, feed: FeedId) {
        self.store.fail_load_more(feed);
        self.mark(feed, |progress| {
            progress.awaiting_first = false;
            progress.awaiting_more = false;
        });
    }

    fn feed_for(&self, context: &FeedContext) -> Option<FeedId> {
        self.feeds
            .iter()
            .find_map(|(feed, progress)| (progress.context == *context).then_some(*feed))
    }

    fn first_loaded(&self, feed: FeedId) -> bool {
        self.feeds
            .get(&feed)
            .is_some_and(|progress| progress.first_loaded)
    }

    fn mark(&mut self, feed: FeedId, update: impl FnOnce(&mut FeedProgress)) {
        if let Some(progress) = self.feeds.get_mut(&feed) {
            update(progress);
        }
    }
}
