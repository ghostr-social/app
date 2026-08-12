use super::{TrafficBatch, TrafficEvent, TrafficWindow, TransferKey, SAMPLE_INTERVAL};
use crate::manager::transfers::InternalEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

mod pending;
use pending::PendingTransfer;

pub(crate) fn channel(
    events: UnboundedSender<InternalEvent>,
    capacity: usize,
) -> (TrafficPublisher, TrafficInbox) {
    let state = Arc::new(Mutex::new(State::new(capacity)));
    (
        TrafficPublisher {
            state: Arc::clone(&state),
            events: events.clone(),
        },
        TrafficInbox { state, events },
    )
}

#[derive(Clone)]
pub(crate) struct TrafficPublisher {
    state: Arc<Mutex<State>>,
    events: UnboundedSender<InternalEvent>,
}

impl TrafficPublisher {
    pub(crate) fn opened(
        &self,
        transfer: TransferKey,
        host: String,
        ttfb: Duration,
        at: Instant,
    ) -> bool {
        let accepted = self.lock().open(transfer, host, ttfb, at);
        if accepted {
            self.wake();
        }
        accepted
    }

    pub(crate) fn progress(&self, transfer: TransferKey, bytes: u64, at: Instant) {
        self.lock().progress(transfer, bytes, at);
    }

    pub(crate) fn closed(&self, transfer: TransferKey, at: Instant) {
        if self.lock().close(transfer, at) {
            self.wake();
        }
    }

    fn wake(&self) {
        let should_wake = self.lock().request_wake();
        if should_wake && self.events.send(InternalEvent::TrafficChanged).is_err() {
            self.lock().wake_pending = false;
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

pub(crate) struct TrafficInbox {
    state: Arc<Mutex<State>>,
    events: UnboundedSender<InternalEvent>,
}

impl TrafficInbox {
    pub(crate) fn drain(&mut self, at: Instant) -> TrafficBatch {
        let (batch, arm_timer) = self.lock().drain(at);
        if arm_timer {
            spawn_timer(Arc::clone(&self.state), self.events.clone(), at);
        }
        batch
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

struct State {
    transfers: HashMap<TransferKey, PendingTransfer>,
    capacity: usize,
    wake_pending: bool,
    timer_armed: bool,
    window_started: Option<Instant>,
}

impl State {
    fn new(capacity: usize) -> Self {
        Self {
            transfers: HashMap::new(),
            capacity,
            wake_pending: false,
            timer_armed: false,
            window_started: None,
        }
    }

    fn open(&mut self, key: TransferKey, host: String, ttfb: Duration, at: Instant) -> bool {
        if self.transfers.contains_key(&key) || self.transfers.len() >= self.capacity {
            return false;
        }
        self.window_started.get_or_insert(at);
        self.transfers
            .insert(key, PendingTransfer::opened(host, ttfb, at));
        true
    }

    fn progress(&mut self, key: TransferKey, bytes: u64, at: Instant) -> bool {
        let Some(transfer) = self.transfers.get_mut(&key) else {
            return false;
        };
        transfer.bytes = transfer.bytes.saturating_add(bytes);
        transfer.last_at = at;
        true
    }

    fn close(&mut self, key: TransferKey, at: Instant) -> bool {
        let Some(transfer) = self.transfers.get_mut(&key) else {
            return false;
        };
        transfer.closed = Some(at);
        transfer.last_at = at;
        true
    }

    fn request_wake(&mut self) -> bool {
        if self.wake_pending {
            return false;
        }
        self.wake_pending = true;
        true
    }

    fn timer_fired(&mut self) -> bool {
        self.timer_armed = false;
        self.has_active() && self.request_wake()
    }

    fn drain(&mut self, at: Instant) -> (TrafficBatch, bool) {
        let mut events = Vec::new();
        let mut latest = self.window_started.unwrap_or(at);
        self.transfers.retain(|key, pending| {
            pending.append_events(*key, &mut events);
            latest = latest.max(pending.last_at);
            pending.reset();
            pending.closed.is_none()
        });
        latest = if self.has_active() {
            latest.max(at)
        } else {
            latest
        };
        let started = self.window_started.unwrap_or(latest);
        self.window_started = (!self.transfers.is_empty()).then_some(latest);
        self.wake_pending = false;
        let arm_timer = self.arm_timer();
        (
            TrafficBatch::new(TrafficWindow::new(started, latest), events),
            arm_timer,
        )
    }

    fn arm_timer(&mut self) -> bool {
        if self.timer_armed || !self.has_active() {
            return false;
        }
        self.timer_armed = true;
        true
    }

    fn has_active(&self) -> bool {
        self.transfers
            .values()
            .any(|transfer| transfer.closed.is_none())
    }
}

fn spawn_timer(state: Arc<Mutex<State>>, events: UnboundedSender<InternalEvent>, started: Instant) {
    tokio::spawn(async move {
        tokio::time::sleep_until(started + SAMPLE_INTERVAL).await;
        let should_wake = lock(&state).timer_fired();
        if should_wake && events.send(InternalEvent::TrafficChanged).is_err() {
            lock(&state).wake_pending = false;
        }
    });
}

fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(|error| error.into_inner())
}
