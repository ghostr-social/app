use super::attempts::TimelineAttempt;
use ghostr_engine::media_timeline::MediaTimeline;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineIncomplete {
    Unavailable,
    Truncated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineRejection {
    Malformed,
    OutOfBounds,
    ResourceLimit,
    Unsupported,
}

#[derive(Debug)]
pub(crate) enum TimelineRetry {
    Missing,
    Read(String),
    Worker(String),
}

#[derive(Debug)]
pub(crate) enum TimelineTerminal {
    Incomplete(TimelineIncomplete),
    Ready(MediaTimeline),
    Rejected(TimelineRejection),
}

#[derive(Debug)]
pub(crate) enum TimelineJobOutcome {
    Retryable(TimelineRetry),
    Superseded,
    Terminal(TimelineTerminal),
}

#[derive(Debug)]
pub(crate) struct TimelineResult {
    pub(super) attempt: TimelineAttempt,
    pub(super) outcome: TimelineJobOutcome,
}

impl TimelineResult {
    pub(super) fn new(attempt: TimelineAttempt, outcome: TimelineJobOutcome) -> Self {
        Self { attempt, outcome }
    }

    pub(crate) fn post(&self) -> &ghostr_engine::PostId {
        self.attempt.post()
    }
}
