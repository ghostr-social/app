use super::TrafficWindow;
use core::time::Duration;
use tokio::time::Instant;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct TransferKey(u64, u64);

impl TransferKey {
    pub(crate) fn next_window(self) -> Self {
        Self(self.0, self.1.checked_add(1).expect("traffic window exhausted"))
    }
    pub(crate) fn new(value: u64) -> Self {
        Self(value, 0)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum TrafficEvent {
    Opened {
        transfer: TransferKey,
        host: String,
        ttfb: Duration,
        at: Instant,
    },
    Resumed { transfer: TransferKey, host: String, at: Instant },
    Progress {
        transfer: TransferKey,
        bytes: u64,
        at: Instant,
    },
    Closed {
        transfer: TransferKey,
        at: Instant,
    },
}

impl TrafficEvent {
    pub(super) fn at(&self) -> Instant {
        match self {
            Self::Opened { at, .. } | Self::Resumed { at, .. } | Self::Progress { at, .. } | Self::Closed { at, .. } => *at,
        }
    }
}

pub(crate) struct TrafficBatch {
    window: TrafficWindow,
    events: Vec<TrafficEvent>,
}

impl TrafficBatch {
    pub(super) fn new(window: TrafficWindow, events: Vec<TrafficEvent>) -> Self {
        Self { window, events }
    }

    pub(super) fn window(&self) -> TrafficWindow {
        self.window
    }

    pub(super) fn events_mut(&mut self) -> &mut [TrafficEvent] {
        &mut self.events
    }

    pub(super) fn into_events(self) -> Vec<TrafficEvent> {
        self.events
    }
}

#[cfg(test)]
#[path = "event_axiom_test.rs"]
pub(crate) mod axiom_test_support;
