use super::context::WatchContext;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchCensor {
    TransportSubstitution,
    OriginFailure,
    PolicyRejection,
    DecodeFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchSampleKind {
    Abandoned,
    Completed,
    Censored(WatchCensor),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WatchSample {
    pub(super) context: WatchContext,
    pub(super) watched_ms: u64,
    pub(super) kind: WatchSampleKind,
    pub(super) observed_at_ms: u64,
}

impl WatchSample {
    pub fn new(
        context: WatchContext,
        watched_ms: u64,
        kind: WatchSampleKind,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            context,
            watched_ms,
            kind,
            observed_at_ms,
        }
    }
}
