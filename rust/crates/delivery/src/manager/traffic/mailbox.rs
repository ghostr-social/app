use super::{TrafficBatch, TrafficEvent, TrafficWindow, TransferKey, SAMPLE_INTERVAL};
use crate::manager::transfers::InternalEvent;
use core::time::Duration;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

mod pending;
mod state;
use pending::PendingTransfer;
use state::State;

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

    pub(crate) fn resumed(&self, transfer: TransferKey, host: String, at: Instant) -> bool {
        let accepted = self.lock().resume(transfer, host, at);
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
    pub(crate) fn drain(&self, at: Instant) -> TrafficBatch {
        let (batch, timer_start) = self.lock().drain(at);
        if let Some(started) = timer_start {
            spawn_timer(Arc::clone(&self.state), self.events.clone(), started);
        }
        batch
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
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
