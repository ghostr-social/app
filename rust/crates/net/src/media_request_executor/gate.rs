use super::MediaRequestLimits;
use anyhow::{Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

mod state;
use state::GateState;

#[derive(Clone)]
pub(super) struct MediaRequestGate {
    inner: Arc<GateInner>,
}

struct GateInner {
    state: Mutex<GateState>,
}

pub(super) struct RequestLease {
    gate: MediaRequestGate,
    authority: RequestAuthority,
    armed: bool,
}

struct QueuedRequest {
    gate: MediaRequestGate,
    sequence: u64,
    armed: bool,
}

impl MediaRequestGate {
    pub(super) fn new(limits: MediaRequestLimits) -> Self {
        Self {
            inner: Arc::new(GateInner {
                state: Mutex::new(GateState::new(limits)),
            }),
        }
    }

    pub(super) async fn acquire(
        &self,
        authority: RequestAuthority,
        priority: PreemptionAuthority,
    ) -> Result<RequestLease> {
        let (granted, receiver) = oneshot::channel();
        let sequence = self.enqueue(authority, priority, granted);
        let mut queued = QueuedRequest::new(self.clone(), sequence);
        let lease = receiver.await.context("media request admission closed")?;
        queued.armed = false;
        Ok(lease)
    }

    pub(super) fn update_limits(&self, limits: MediaRequestLimits) {
        self.with_state(|state| state.limits = limits);
        self.dispatch();
    }

    pub(super) fn limits(&self) -> MediaRequestLimits {
        self.with_state(|state| state.limits)
    }

    pub(super) fn active_for(&self, authority: &RequestAuthority) -> usize {
        self.with_state(|state| state.active_for(authority))
    }

    pub(super) fn active_connections(&self) -> Vec<(String, usize)> {
        self.with_state(|state| state.active_connections())
    }

    fn enqueue(
        &self,
        authority: RequestAuthority,
        priority: PreemptionAuthority,
        granted: oneshot::Sender<RequestLease>,
    ) -> u64 {
        let sequence = self.with_state(|state| state.enqueue(authority, priority, granted));
        self.dispatch();
        sequence
    }

    fn cancel(&self, sequence: u64) {
        self.with_state(|state| state.cancel(sequence));
        self.dispatch();
    }

    fn release(&self, authority: &RequestAuthority) {
        self.with_state(|state| state.release(authority));
        self.dispatch();
    }

    fn dispatch(&self) {
        loop {
            let failed = self.dispatch_once();
            if failed.is_empty() {
                break;
            }
            self.release_failed(&failed);
        }
    }

    fn dispatch_once(&self) -> Vec<RequestAuthority> {
        self.with_state(|state| state.take_grants(self))
            .into_iter()
            .filter_map(|(sender, lease)| returned_authority(sender.send(lease)))
            .collect()
    }

    fn release_failed(&self, authorities: &[RequestAuthority]) {
        self.with_state(|state| {
            for authority in authorities {
                state.release(authority);
            }
        });
    }

    fn with_state<T>(&self, operation: impl FnOnce(&mut GateState) -> T) -> T {
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut state)
    }
}

fn returned_authority(result: Result<(), RequestLease>) -> Option<RequestAuthority> {
    result.err().map(|mut lease| {
        lease.armed = false;
        lease.authority.clone()
    })
}

impl RequestLease {
    fn new(gate: MediaRequestGate, authority: RequestAuthority) -> Self {
        Self {
            gate,
            authority,
            armed: true,
        }
    }
}

impl Drop for RequestLease {
    fn drop(&mut self) {
        if self.armed {
            self.gate.release(&self.authority);
        }
    }
}

impl QueuedRequest {
    fn new(gate: MediaRequestGate, sequence: u64) -> Self {
        Self {
            gate,
            sequence,
            armed: true,
        }
    }
}

impl Drop for QueuedRequest {
    fn drop(&mut self) {
        if self.armed {
            self.gate.cancel(self.sequence);
        }
    }
}
