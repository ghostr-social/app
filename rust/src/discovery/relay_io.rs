//! External nostr-sdk network calls behind the relay-pool owner.

use anyhow::Context;
use log::warn;
use nostr_sdk::{Client, Event, Filter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};

pub(crate) type RelayIoFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;

pub(crate) struct RelayReadIo {
    pub relays: Vec<String>,
    pub filter: Filter,
    pub timeout: Duration,
}

pub(crate) struct RelayBroadcastIo {
    pub relays: Vec<String>,
    pub event: Event,
}

pub(crate) trait RelayIo: Send + Sync {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>>;
    fn broadcast(&self, request: RelayBroadcastIo) -> RelayIoFuture<'_, ()>;
}

pub(crate) struct SdkRelayIo {
    client: Arc<Client>,
}

impl SdkRelayIo {
    pub(crate) fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    async fn connect(&self, relays: &[String]) {
        for relay in relays {
            if let Err(error) = self.client.connect_relay(relay).await {
                warn!("Nostr relay {relay} could not connect: {error}");
            }
        }
    }
}

impl RelayIo for SdkRelayIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move {
            self.connect(&request.relays).await;
            let stream = self
                .client
                .stream_events_from(request.relays, vec![request.filter], request.timeout)
                .await
                .context("relay query failed")?;
            Ok(drain_events(stream).await)
        })
    }

    fn broadcast(&self, request: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async move {
            self.connect(&request.relays).await;
            self.client
                .send_event_to(request.relays, request.event)
                .await
                .context("broadcast failed")?;
            Ok(())
        })
    }
}

pub(crate) async fn drain_events<S>(mut stream: S) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }
    events
}
