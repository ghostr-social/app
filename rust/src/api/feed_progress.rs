//! What one open feed is currently waiting for. Owned by `FeedState`,
//! which turns it into the [`FfiFeedStage`] every snapshot carries, so
//! the pull-shaped Dart adapter can stop waiting the moment a page is
//! genuinely complete instead of guessing from the row count.

use crate::api::feed_types::FfiFeedStage;
use crate::discovery::retrieval_queue::FeedContext;
use flutter_rust_bridge::frb;

#[frb(ignore)]
#[derive(Debug)]
pub(crate) struct FeedProgress {
    pub(crate) context: FeedContext,
    pub(crate) first_loaded: bool,
    awaiting_first: bool,
    awaiting_more: bool,
    failed: bool,
}

impl FeedProgress {
    /// A feed whose open dispatched no query has nothing to wait for
    /// (blank search parity) and is settled from the start.
    pub(crate) fn new(context: FeedContext, awaiting_first: bool) -> Self {
        Self {
            context,
            first_loaded: false,
            awaiting_first,
            awaiting_more: false,
            failed: false,
        }
    }

    pub(crate) fn stage(&self) -> FfiFeedStage {
        if self.awaiting_first || self.awaiting_more {
            return FfiFeedStage::Loading;
        }
        if self.failed {
            return FfiFeedStage::Failed;
        }
        FfiFeedStage::Settled
    }

    pub(crate) fn is_awaiting(&self) -> bool {
        self.awaiting_first || self.awaiting_more
    }

    pub(crate) fn await_first(&mut self) {
        self.awaiting_first = true;
        self.failed = false;
    }

    pub(crate) fn await_more(&mut self) {
        self.awaiting_more = true;
        self.failed = false;
    }

    pub(crate) fn record_page(&mut self) {
        self.first_loaded = true;
        self.settle(false);
    }

    pub(crate) fn record_failure(&mut self) {
        self.settle(true);
    }

    fn settle(&mut self, failed: bool) {
        self.awaiting_first = false;
        self.awaiting_more = false;
        self.failed = failed;
    }
}
