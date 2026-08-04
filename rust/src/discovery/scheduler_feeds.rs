//! Per-context feed bookkeeping behind the discovery scheduler: base
//! request, older-page cursor, and in-flight accounting — everything
//! the control-loop policy asks about the active feed.

use crate::discovery::control_loop::FeedQueryState;
use crate::discovery::retrieval_queue::FeedContext;
use crate::discovery::video_filters::DiscoveryRequest;
use nostr_sdk::Timestamp;
use std::collections::HashMap;

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
}

impl FeedBook {
    /// Opens (or reopens) a feed with a fresh page history and makes
    /// it the active one.
    pub(crate) fn open(&mut self, context: FeedContext, request: DiscoveryRequest) {
        self.feeds.insert(
            context.clone(),
            FeedProgress {
                request,
                cursor: None,
                loaded: false,
                widened: false,
            },
        );
        self.active = Some(context);
    }

    pub(crate) fn active(&self) -> Option<&FeedContext> {
        self.active.as_ref()
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
    pub(crate) fn record_page(&mut self, context: &FeedContext, cursor: Option<Timestamp>) {
        if let Some(feed) = self.feeds.get_mut(context) {
            feed.loaded = true;
            feed.cursor = cursor;
        }
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
