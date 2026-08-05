//! Per-context request, cursor, and in-flight discovery bookkeeping.

use crate::discovery::control_loop::FeedQueryState;
use crate::discovery::retrieval_queue::FeedContext;
use crate::discovery::video_filters::DiscoveryRequest;
use nostr_sdk::Timestamp;
use std::collections::HashMap;
use std::time::Duration;

pub(crate) const QUERY_HUNT_BACKOFF: Duration = Duration::from_secs(8);
const QUERY_HUNT_PAGE_BURST: u8 = 3;

pub(crate) enum QueryHuntAction {
    OlderNow,
    HeadLater,
}

#[derive(Debug, Default)]
pub(crate) struct FeedBook {
    active: Option<FeedContext>,
    feeds: HashMap<FeedContext, FeedProgress>,
    inflight: HashMap<FeedContext, usize>,
}

#[derive(Debug)]
struct FeedProgress {
    request: DiscoveryRequest,
    cursor: Option<Timestamp>,
    loaded: bool,
    widened: bool,
    query: bool,
    older_pages: u8,
    failed: bool,
    playable: bool,
}

impl FeedBook {
    /// Opens (or reopens) a feed with a fresh page history and makes
    /// it the active one.
    pub(crate) fn open(&mut self, context: FeedContext, request: DiscoveryRequest) {
        let query = request.is_wide();
        self.feeds.insert(
            context.clone(),
            FeedProgress {
                request,
                cursor: None,
                loaded: false,
                widened: false,
                query,
                older_pages: 0,
                failed: false,
                playable: false,
            },
        );
        self.active = Some(context);
    }

    pub(crate) fn active(&self) -> Option<&FeedContext> {
        self.active.as_ref()
    }

    pub(crate) fn reset_session(&mut self) {
        self.active = None;
        self.feeds.clear();
        self.inflight.clear();
    }

    pub(crate) fn close(&mut self, context: &FeedContext) {
        self.feeds.remove(context);
        self.inflight.remove(context);
        if self.active.as_ref() == Some(context) {
            self.active = None;
        }
    }

    pub(crate) fn base_request(&self, context: &FeedContext) -> Option<&DiscoveryRequest> {
        self.feeds.get(context).map(|feed| &feed.request)
    }

    /// The stored request aimed at the next older page. `None` without
    /// an explicit or tracked cursor: nothing older is known.
    pub(crate) fn older_page_request(
        &self,
        context: &FeedContext,
        older_than: Option<Timestamp>,
    ) -> Option<DiscoveryRequest> {
        let feed = self.feeds.get(context)?;
        let cursor = older_than.or(feed.cursor)?;
        Some(DiscoveryRequest {
            older_than: Some(cursor),
            ..feed.request.clone()
        })
    }

    pub(crate) fn mark_widened(&mut self, context: &FeedContext) {
        if let Some(feed) = self.feeds.get_mut(context) {
            feed.widened = true;
        }
    }

    pub(crate) fn record_start(&mut self, context: &FeedContext) {
        *self.inflight.entry(context.clone()).or_default() += 1;
    }

    pub(crate) fn record_done(&mut self, context: &FeedContext) {
        if let Some(count) = self.inflight.get_mut(context) {
            *count = count.saturating_sub(1);
        }
    }

    /// A landed page marks the feed loaded; a `None` cursor after a
    /// load means the feed looks exhausted.
    pub(crate) fn record_page(
        &mut self,
        context: &FeedContext,
        cursor: Option<Timestamp>,
        head: bool,
    ) {
        if let Some(feed) = self.feeds.get_mut(context) {
            let first = !feed.loaded;
            feed.loaded = true;
            feed.failed = false;
            feed.playable |= cursor.is_some();
            if first || !head {
                feed.cursor = cursor;
            }
            if !head {
                feed.older_pages = feed.older_pages.saturating_add(1);
            }
        }
    }

    pub(crate) fn record_failure(&mut self, context: &FeedContext) {
        if let Some(feed) = self.feeds.get_mut(context) {
            feed.loaded = true;
            feed.failed = true;
        }
    }

    pub(crate) fn record_playable(&mut self, context: &FeedContext) {
        if let Some(feed) = self.feeds.get_mut(context) {
            feed.playable = true;
        }
    }

    pub(crate) fn has_playable(&self, context: &FeedContext) -> bool {
        self.feeds.get(context).is_some_and(|feed| feed.playable)
    }

    pub(crate) fn head_request(&self, context: &FeedContext) -> Option<DiscoveryRequest> {
        let feed = self.feeds.get(context)?;
        Some(DiscoveryRequest {
            older_than: None,
            ..feed.request.clone()
        })
    }

    pub(crate) fn is_query(&self, context: &FeedContext) -> bool {
        self.feeds.get(context).is_some_and(|feed| feed.query)
    }

    pub(crate) fn hunt_action(&mut self, context: &FeedContext) -> Option<QueryHuntAction> {
        let feed = self.feeds.get_mut(context)?;
        if !feed.query || self.inflight.get(context).copied().unwrap_or(0) > 0 {
            return None;
        }
        if feed.failed || feed.cursor.is_none() {
            feed.failed = false;
            return Some(QueryHuntAction::HeadLater);
        }
        if feed.older_pages >= QUERY_HUNT_PAGE_BURST {
            feed.older_pages = 0;
            return Some(QueryHuntAction::HeadLater);
        }
        Some(QueryHuntAction::OlderNow)
    }

    pub(crate) fn total_inflight(&self) -> usize {
        self.inflight.values().sum()
    }

    /// The control-loop picture of one feed; `queued` comes from the
    /// retrieval queue.
    pub(crate) fn query_state(&self, context: &FeedContext, queued: bool) -> FeedQueryState {
        let busy = queued || self.inflight.get(context).copied().unwrap_or(0) > 0;
        match self.feeds.get(context) {
            None => FeedQueryState {
                busy,
                ..FeedQueryState::default()
            },
            Some(feed) => FeedQueryState {
                open: true,
                busy,
                has_cursor: feed.cursor.is_some(),
                loaded: feed.loaded,
                widened: feed.widened,
            },
        }
    }
}
