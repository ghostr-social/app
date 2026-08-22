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
    pub(crate) context: WatchContext,
    pub(crate) watched_ms: u64,
    pub(crate) kind: WatchSampleKind,
    pub(crate) observed_at_ms: u64,
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
