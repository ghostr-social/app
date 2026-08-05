//! Controllable external relay IO for lifecycle ownership tests.

use crate::discovery::relay_io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::discovery::relay_pool_owner::RelayReadRequest;
use crate::discovery::search_queries::{OutboxRoute, PlannedQuery, QueryRole, RelayTarget};
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::{Event, Filter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{Notify, Semaphore};

pub(crate) struct TestRelayIo {
    query_gate: Semaphore,
    send_gate: Semaphore,
    pub(crate) query_started: Notify,
    pub(crate) send_started: Notify,
    reads: AtomicUsize,
    sends: AtomicUsize,
}

impl TestRelayIo {
    pub(crate) fn blocked() -> Self {
        Self {
            query_gate: Semaphore::new(0),
            send_gate: Semaphore::new(0),
            query_started: Notify::new(),
            send_started: Notify::new(),
            reads: AtomicUsize::new(0),
            sends: AtomicUsize::new(0),
        }
    }

    pub(crate) fn release_query(&self) {
        self.query_gate.add_permits(1);
    }

    pub(crate) fn release_send(&self) {
        self.send_gate.add_permits(1);
    }

    pub(crate) fn send_count(&self) -> usize {
        self.sends.load(Ordering::SeqCst)
    }

    pub(crate) fn read_count(&self) -> usize {
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

pub(crate) fn read_request(relay: &str) -> RelayReadRequest {
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
