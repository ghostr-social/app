//! API-side bookkeeping of every open feed: routes retrieval outcomes
//! into the `FeedStore`, claims load-more cursors, and builds full row
//! snapshots. Pure state — the FFI runtime wraps it in a lock and
//! forwards the returned dispatches to the discovery scheduler.

use crate::api::feed_decisions::{LoadMoreAction, LoadMoreDecision, OpenDispatch};
use crate::api::feed_mapping::{feed_post, resolved_creator};
use crate::api::feed_progress::FeedProgress;
use crate::api::feed_types::{FfiFeedPost, FfiFeedStage};
use crate::discovery::event_parsing::{video_post_from_event, ParsedVideoPost};
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::feed_store::{FeedId, FeedStore};
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::profile_store::ProfileStore;
use crate::discovery::retrieval_queue::FeedContext;
use crate::discovery::session_generation::SessionGeneration;
use crate::discovery::social_graph::SocialGraph;
use flutter_rust_bridge::frb;
use nostr_sdk::{Event, Keys, Timestamp};
use std::collections::HashMap;
use tokio::sync::watch;

mod session;

/// Every open feed plus the shared profile store and social graph. The
/// graph belongs to the newest signed-in main-feed viewer; a throwaway
/// key stands in until one opens, so nothing is muted or follow-routed.
#[frb(ignore)]
#[derive(Debug)]
pub(crate) struct FeedState {
    store: FeedStore,
    profiles: ProfileStore,
    graph: SocialGraph,
    feeds: HashMap<FeedId, FeedProgress>,
    session: SessionGeneration,
}

impl FeedState {
    pub(crate) fn new() -> Self {
        Self {
            store: FeedStore::new(),
            profiles: ProfileStore::new(),
            graph: SocialGraph::new(Keys::generate().public_key()),
            feeds: HashMap::new(),
            session: SessionGeneration::initial(),
        }
    }

    /// Opens the feed and returns its first-page dispatch; `None` when
    /// the spec can never produce content (blank search parity).
    pub(crate) fn open(&mut self, spec: FeedSpec) -> (FeedId, Option<OpenDispatch>) {
        self.adopt_viewer(&spec);
        let feed = self.store.open_feed(spec.clone());
        let context = FeedContext::for_session(format!("feed-{}", feed.0), self.session);
        let dispatch = spec
            .page_request(None, &self.graph)
            .map(|request| OpenDispatch {
                context: context.clone(),
                request,
            });
        self.feeds
            .insert(feed, FeedProgress::new(context, dispatch.is_some()));
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
        if progress.is_awaiting() {
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

    /// How far the feed's current page got; a feed nobody opened has
    /// nothing left in flight.
    pub(crate) fn stage(&self, feed: FeedId) -> FfiFeedStage {
        self.feeds
            .get(&feed)
            .map_or(FfiFeedStage::Settled, FeedProgress::stage)
    }

    /// The feed's visible rows, newest first, creators resolved.
    pub(crate) fn snapshot(&self, feed: FeedId) -> Vec<FfiFeedPost> {
        self.store
            .posts(feed)
            .iter()
            .map(|post| feed_post(post, resolved_creator(&self.profiles, post)))
            .collect()
    }

    fn reopen(&mut self, feed: FeedId, context: FeedContext) -> LoadMoreDecision {
        let spec = self.store.spec(feed);
        let Some(request) = spec.and_then(|spec| spec.page_request(None, &self.graph)) else {
            return LoadMoreDecision::finished();
        };
        self.mark(feed, FeedProgress::await_first);
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
        self.mark(feed, FeedProgress::await_more);
        LoadMoreDecision {
            may_have_more: true,
            action: LoadMoreAction::Older {
                context,
                older_than: explicit.unwrap_or(cursor),
            },
        }
    }

    /// The store notifies for the rows; the stage moving out of
    /// `Loading` is a snapshot change of its own, so a page that added
    /// nothing still publishes a revision.
    fn ingest_page(&mut self, feed: FeedId, events: &[Event]) {
        for event in events {
            self.profiles.ingest(event);
        }
        let posts: Vec<ParsedVideoPost> = events.iter().filter_map(video_post_from_event).collect();
        let published = if self.feeds.get(&feed).is_some_and(|it| it.first_loaded) {
            self.store.ingest_older_page(feed, posts, &self.graph)
        } else {
            self.store.ingest_first_page(feed, posts, &self.graph);
            true
        };
        self.mark(feed, FeedProgress::record_page);
        if !published {
            self.store.touch(feed);
        }
    }

    fn record_failure(&mut self, feed: FeedId) {
        self.store.fail_load_more(feed);
        self.mark(feed, FeedProgress::record_failure);
        self.store.touch(feed);
    }

    fn feed_for(&self, context: &FeedContext) -> Option<FeedId> {
        self.feeds
            .iter()
            .find_map(|(feed, progress)| (progress.context == *context).then_some(*feed))
    }

    fn mark(&mut self, feed: FeedId, update: impl FnOnce(&mut FeedProgress)) {
        if let Some(progress) = self.feeds.get_mut(&feed) {
            update(progress);
        }
    }
}
