//! Unified control loop policy (plan §5.4): the delivery engine's
//! inventory mode steers discovery. Hunger widens the active feed's
//! querying; comfort never spends the radio speculatively. Pure and
//! table-tested — the scheduler only executes the returned action.

use ghostr_engine::inventory_controller::Mode;

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

/// Speculative discovery work one mode transition may trigger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryAction {
    /// Nothing speculative.
    Idle,
    /// Fetch the next older page ahead of the viewer.
    PrefetchNextPage,
    /// Re-issue the active feed's query at the wide limit.
    WidenActiveQuery,
}

/// Hunger prefetches when a cursor is known and widens once when the
/// feed looks exhausted; comfort always stays idle.
pub(crate) fn discovery_action(mode: Mode, feed: FeedQueryState) -> DiscoveryAction {
    match mode {
        Mode::Comfort => DiscoveryAction::Idle,
        Mode::Hunger => hunger_action(feed),
    }
}

fn hunger_action(feed: FeedQueryState) -> DiscoveryAction {
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
