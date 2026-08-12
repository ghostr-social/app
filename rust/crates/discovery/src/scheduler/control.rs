//! Adaptive candidate demand steers speculative discovery. Expansion
//! widens the active feed; hold keeps the radio quiet.

use ghostr_engine::adaptive::DiscoveryDemand;

/// What the scheduler knows about the active feed when a mode
/// transition lands.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FeedQueryState {
    /// An open feed context exists.
    pub(crate) open: bool,
    /// A retrieval for that context is queued or running.
    pub(crate) busy: bool,
    /// An older-page cursor is known (the last page was non-empty).
    pub(crate) has_cursor: bool,
    /// At least one page has landed.
    pub(crate) loaded: bool,
    /// The feed's query was already widened once since it opened.
    pub(crate) widened: bool,
}

/// Speculative discovery work one demand transition may trigger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryAction {
    /// Nothing speculative.
    Idle,
    /// Fetch the next older page ahead of the viewer.
    PrefetchNextPage,
    /// Re-issue the active feed's query at the wide limit.
    WidenActiveQuery,
}

/// Expansion prefetches when a cursor is known and widens once when the
/// feed looks exhausted; held demand always stays idle.
pub(crate) fn discovery_action(demand: DiscoveryDemand, feed: FeedQueryState) -> DiscoveryAction {
    match demand {
        DiscoveryDemand::Hold => DiscoveryAction::Idle,
        DiscoveryDemand::Expand => expansion_action(feed),
    }
}

fn expansion_action(feed: FeedQueryState) -> DiscoveryAction {
    if !feed.available() {
        return DiscoveryAction::Idle;
    }
    if feed.has_cursor {
        return DiscoveryAction::PrefetchNextPage;
    }
    if feed.can_widen() {
        return DiscoveryAction::WidenActiveQuery;
    }
    DiscoveryAction::Idle
}

impl FeedQueryState {
    fn available(self) -> bool {
        self.open && !self.busy
    }

    fn can_widen(self) -> bool {
        self.loaded && !self.widened
    }
}
