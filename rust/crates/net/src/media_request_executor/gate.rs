use super::{MediaRequestLimits, MediaResourceObserver};
use anyhow::{Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

mod state;
use state::GateState;
mod observer;
use observer::ResourceObserverSlot;

#[derive(Clone)]
pub(super) struct MediaRequestGate {
    inner: Arc<GateInner>,
}

struct GateInner {
    state: Mutex<GateState>,
    observer: ResourceObserverSlot,
}

pub(super) struct RequestLease {
    gate: MediaRequestGate,
    authority: RequestAuthority,
    priority: PreemptionAuthority,
    armed: bool,
}

struct ReleasedRequest {
    authority: RequestAuthority,
    priority: PreemptionAuthority,
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
                observer: ResourceObserverSlot::default(),
            }),
        }
    }

    pub(super) fn install_resource_observer(
        &self,
        observer: Arc<dyn MediaResourceObserver>,
    ) -> bool {
        self.inner.observer.install(observer)
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

    fn release(&self, authority: &RequestAuthority, priority: PreemptionAuthority) {
        self.with_state(|state| state.release(authority, priority));
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

    fn dispatch_once(&self) -> Vec<ReleasedRequest> {
        self.with_state(|state| state.take_grants(self))
            .into_iter()
            .filter_map(|(sender, lease)| returned_request(sender.send(lease)))
            .collect()
    }

    fn release_failed(&self, requests: &[ReleasedRequest]) {
        self.with_state(|state| {
            for request in requests {
                state.release(&request.authority, request.priority);
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

fn returned_request(result: Result<(), RequestLease>) -> Option<ReleasedRequest> {
    result.err().map(|mut lease| {
        lease.armed = false;
        ReleasedRequest {
            authority: lease.authority.clone(),
            priority: lease.priority,
        }
    })
}

impl RequestLease {
    fn new(
        gate: MediaRequestGate,
        authority: RequestAuthority,
        priority: PreemptionAuthority,
    ) -> Self {
        Self {
            gate,
            authority,
            priority,
            armed: true,
        }
    }
}

impl Drop for RequestLease {
    fn drop(&mut self) {
        if self.armed {
            self.gate.release(&self.authority, self.priority);
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
