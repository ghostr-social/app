use super::{Duration, Instant, TrafficEvent, TransferKey};

pub(super) struct PendingTransfer {
    host: String,
    opened: Option<(Duration, Instant)>,
    pub(super) bytes: u64,
    pub(super) last_at: Instant,
    pub(super) closed: Option<Instant>,
}

impl PendingTransfer {
    pub(super) fn opened(host: String, ttfb: Duration, at: Instant) -> Self {
        Self {
            host,
            opened: Some((ttfb, at)),
            bytes: 0,
            last_at: at,
            closed: None,
        }
    }

    pub(super) fn append_events(&self, key: TransferKey, events: &mut Vec<TrafficEvent>) {
        if let Some((ttfb, at)) = self.opened {
            events.push(TrafficEvent::Opened {
                transfer: key,
                host: self.host.clone(),
                ttfb,
                at,
            });
        }
        if self.bytes > 0 {
            events.push(TrafficEvent::Progress {
                transfer: key,
                bytes: self.bytes,
                at: self.last_at,
            });
        }
        if let Some(at) = self.closed {
            events.push(TrafficEvent::Closed { transfer: key, at });
        }
    }

    pub(super) fn reset(&mut self) {
        self.opened = None;
        self.bytes = 0;
    }
}
