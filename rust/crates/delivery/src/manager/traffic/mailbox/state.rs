use super::{Duration, Instant, PendingTransfer, TrafficBatch, TrafficWindow, TransferKey};
use std::collections::HashMap;

pub(super) struct State {
    transfers: HashMap<TransferKey, PendingTransfer>,
    capacity: usize,
    pub(super) wake_pending: bool,
    timer_armed: bool,
    window_started: Option<Instant>,
}

impl State {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            transfers: HashMap::new(),
            capacity,
            wake_pending: false,
            timer_armed: false,
            window_started: None,
        }
    }

    pub(super) fn open(
        &mut self,
        key: TransferKey,
        host: String,
        ttfb: Duration,
        at: Instant,
    ) -> bool {
        if self.transfers.contains_key(&key) || self.transfers.len() >= self.capacity {
            return false;
        }
        self.window_started.get_or_insert(at);
        self.transfers
            .insert(key, PendingTransfer::opened(host, ttfb, at));
        true
    }

    pub(super) fn resume(&mut self, key: TransferKey, host: String, at: Instant) -> bool {
        if !self.open(key, host, Duration::ZERO, at) {
            return false;
        }
        self.transfers
            .get_mut(&key)
            .expect("opened traffic window")
            .resumed = true;
        true
    }

    pub(super) fn progress(&mut self, key: TransferKey, bytes: u64, at: Instant) -> bool {
        let Some(transfer) = self.transfers.get_mut(&key) else {
            return false;
        };
        transfer.bytes = transfer.bytes.saturating_add(bytes);
        transfer.last_at = at;
        true
    }

    pub(super) fn close(&mut self, key: TransferKey, at: Instant) -> bool {
        let Some(transfer) = self.transfers.get_mut(&key) else {
            return false;
        };
        transfer.closed = Some(at);
        transfer.last_at = at;
        true
    }

    pub(super) fn request_wake(&mut self) -> bool {
        if self.wake_pending {
            return false;
        }
        self.wake_pending = true;
        true
    }

    pub(super) fn timer_fired(&mut self) -> bool {
        self.timer_armed = false;
        self.request_wake()
    }

    pub(super) fn drain(&mut self, at: Instant) -> (TrafficBatch, Option<Instant>) {
        let mut events = Vec::new();
        let mut latest = self.window_started.unwrap_or(at);
        self.transfers.retain(|key, pending| {
            pending.append_events(*key, &mut events);
            latest = latest.max(pending.last_at);
            pending.reset();
            pending.closed.is_none()
        });
        if self.has_active() {
            latest = latest.max(at);
        }
        let started = self.window_started.unwrap_or(latest);
        self.window_started = (!self.transfers.is_empty()).then_some(latest);
        self.wake_pending = false;
        let timer_start = self.timer_start(started, latest, !events.is_empty());
        (
            TrafficBatch::new(TrafficWindow::new(started, latest), events),
            timer_start,
        )
    }

    fn timer_start(
        &mut self,
        started: Instant,
        latest: Instant,
        had_events: bool,
    ) -> Option<Instant> {
        if self.timer_armed || (!self.has_active() && !had_events) {
            return None;
        }
        self.timer_armed = true;
        Some(if self.has_active() { latest } else { started })
    }

    fn has_active(&self) -> bool {
        self.transfers.values().any(|item| item.closed.is_none())
    }
}
