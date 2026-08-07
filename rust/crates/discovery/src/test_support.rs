//! Controllable external relay IO for lifecycle ownership tests.
//!
//! Gated behind the `test-support` feature so dependents can drive the relay
//! pool with a scripted IO port instead of reaching a live relay.

use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::relay::pool::RelayReadRequest;
use crate::query::search::{OutboxRoute, PlannedQuery, QueryRole, RelayTarget};
use crate::session_generation::SessionGeneration;
use nostr_sdk::{Event, Filter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{Notify, Semaphore};

pub struct TestRelayIo {
    query_gate: Semaphore,
    send_gate: Semaphore,
    pub query_started: Notify,
    pub send_started: Notify,
    reads: AtomicUsize,
    sends: AtomicUsize,
}

impl TestRelayIo {
    pub fn blocked() -> Self {
        Self {
            query_gate: Semaphore::new(0),
            send_gate: Semaphore::new(0),
            query_started: Notify::new(),
            send_started: Notify::new(),
            reads: AtomicUsize::new(0),
            sends: AtomicUsize::new(0),
        }
    }

    pub fn release_query(&self) {
        self.query_gate.add_permits(1);
    }

    pub fn release_send(&self) {
        self.send_gate.add_permits(1);
    }

    pub fn send_count(&self) -> usize {
        self.sends.load(Ordering::SeqCst)
    }

    pub fn read_count(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }

    async fn pass(gate: &Semaphore) -> anyhow::Result<()> {
        gate.acquire().await?.forget();
        Ok(())
    }
}

impl RelayIo for TestRelayIo {
    fn read(&self, _request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move {
            self.reads.fetch_add(1, Ordering::SeqCst);
            self.query_started.notify_one();
            Self::pass(&self.query_gate).await?;
            Ok(Vec::new())
        })
    }

    fn broadcast(&self, _request: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async move {
            self.sends.fetch_add(1, Ordering::SeqCst);
            self.send_started.notify_one();
            Self::pass(&self.send_gate).await
        })
    }
}

pub fn read_request(relay: &str) -> RelayReadRequest {
    RelayReadRequest {
        session: SessionGeneration::initial(),
        relays: Some(vec![relay.to_owned()]),
        query: PlannedQuery {
            filter: Filter::new(),
            target: RelayTarget::OutboxRelays,
            role: QueryRole::Primary,
            timeout: Duration::from_secs(5),
            outbox: OutboxRoute::Shared,
        },
        progress: None,
    }
}
