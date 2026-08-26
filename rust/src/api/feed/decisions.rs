//! Typed dispatch instructions `FeedState` hands back to the FFI
//! runtime, which forwards them to the discovery scheduler outside the
//! state lock.

use crate::discovery::query::video_filters::DiscoveryRequest;
use crate::discovery::retrieval_types::FeedContext;
use flutter_rust_bridge::frb;
use nostr_sdk::Timestamp;

/// A first-page query to send to the scheduler.
#[frb(ignore)]
#[derive(Clone, Debug)]
pub(crate) struct OpenDispatch {
    pub context: FeedContext,
    pub request: DiscoveryRequest,
}

/// What a load-more call should send to the scheduler, if anything.
#[frb(ignore)]
#[derive(Clone, Debug)]
pub(crate) enum LoadMoreAction {
    None,
    /// The first page never landed: retry the opening query.
    Reopen(OpenDispatch),
    /// One claimed older page; an explicit FFI cursor already won here.
    Older {
        context: FeedContext,
        older_than: Timestamp,
    },
}

#[frb(ignore)]
#[derive(Clone, Debug)]
pub(crate) struct LoadMoreDecision {
    /// Whether more content may exist (`ffi_load_more`'s return value).
    pub may_have_more: bool,
    pub action: LoadMoreAction,
}

impl LoadMoreDecision {
    pub(super) fn finished() -> Self {
        Self {
            may_have_more: false,
            action: LoadMoreAction::None,
        }
    }

    pub(super) fn wait() -> Self {
        Self {
            may_have_more: true,
            action: LoadMoreAction::None,
        }
    }
}
