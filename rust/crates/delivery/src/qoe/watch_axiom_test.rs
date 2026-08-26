use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WatchOutcome {
    watched_ms: u64,
    kind: WatchSampleKind,
}

impl WatchOutcome {
    pub(crate) const fn sample(watched_ms: u64, kind: WatchSampleKind) -> Self {
        Self { watched_ms, kind }
    }
}

impl WatchLearner {
    pub(crate) fn last_outcome(&self) -> Option<WatchOutcome> {
        self.last_outcome
    }
    pub(crate) fn last_navigation(&self) -> Option<WatchNavigation> {
        self.last_navigation
    }
}
