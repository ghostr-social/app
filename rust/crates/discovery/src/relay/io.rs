//! External nostr-sdk network calls behind the relay-pool owner.

use anyhow::{bail, Context};
use log::warn;
use nostr_sdk::pool::RelayNotification;
use nostr_sdk::{Client, Event, Filter, RelayStatus};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};
use tokio_stream::{Stream, StreamExt};

use crate::retrieval_types::EventProgress;

const RELAY_READINESS_TIMEOUT: Duration = Duration::from_secs(15);

pub type RelayIoFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;

pub struct RelayReadIo {
    pub(crate) relays: Vec<String>,
    pub(crate) filter: Filter,
    pub(crate) timeout: Duration,
    pub(crate) progress: Option<EventProgress>,
}

pub struct RelayBroadcastIo {
    pub(crate) relays: Vec<String>,
    pub(crate) event: Event,
}

pub trait RelayIo: Send + Sync {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>>;
    fn broadcast(&self, request: RelayBroadcastIo) -> RelayIoFuture<'_, ()>;
}

pub(crate) struct SdkRelayIo {
    client: Arc<Client>,
    readiness_timeout: Duration,
}

impl SdkRelayIo {
    pub(crate) fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            readiness_timeout: RELAY_READINESS_TIMEOUT,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_readiness_timeout(client: Arc<Client>, readiness_timeout: Duration) -> Self {
        Self {
            client,
            readiness_timeout,
        }
    }

    async fn await_connected_target(&self, relays: &[String]) -> anyhow::Result<()> {
        let mut waiters = JoinSet::new();
        for relay in relays {
            waiters.spawn(wait_for_connection(self.client.clone(), relay.clone()));
        }
        let connected = timeout(self.readiness_timeout, first_connected(&mut waiters))
            .await
            .unwrap_or(false);
        if !connected {
            bail!("no target relay connected before the read deadline");
        }
        Ok(())
    }
}

impl RelayIo for SdkRelayIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move {
            self.await_connected_target(&request.relays).await?;
            let deadline = request.timeout.saturating_add(Duration::from_secs(1));
            let stream = self
                .client
                .stream_events_from(request.relays, vec![request.filter], request.timeout)
                .await
                .context("relay query failed")?;
            Ok(drain_events_until(stream, request.progress, deadline).await)
        })
    }

    fn broadcast(&self, request: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async move {
            for relay in &request.relays {
                if let Err(error) = self.client.connect_relay(relay).await {
                    warn!("Nostr relay {relay} could not connect: {error}");
                }
            }
            self.await_connected_target(&request.relays)
                .await
                .context("broadcast failed")?;
            self.client
                .send_event_to(request.relays, request.event)
                .await
                .context("broadcast failed")?;
            Ok(())
        })
    }
}

async fn first_connected(waiters: &mut JoinSet<bool>) -> bool {
    while let Some(result) = waiters.join_next().await {
        if result.unwrap_or(false) {
            return true;
        }
    }
    false
}

async fn wait_for_connection(client: Arc<Client>, url: String) -> bool {
    let Ok(relay) = client.relay(&url).await else {
        return false;
    };
    let mut notifications = relay.notifications();
    if relay.status() == RelayStatus::Connected {
        return true;
    }
    relay.connect(None).await;
    loop {
        if relay.status() == RelayStatus::Connected {
            return true;
        }
        match notifications.recv().await {
            Ok(RelayNotification::RelayStatus {
                status: RelayStatus::Connected,
            }) => return true,
            Ok(RelayNotification::Shutdown) | Err(RecvError::Closed) => return false,
            Ok(_) | Err(RecvError::Lagged(_)) => {}
        }
    }
}

#[cfg(test)]
pub(crate) async fn drain_events<S>(mut stream: S) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    drain_events_with_progress(&mut stream, None).await
}

#[cfg(test)]
pub(crate) async fn drain_events_with_progress<S>(
    mut stream: S,
    progress: Option<EventProgress>,
) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        if let Some(progress) = &progress {
            let _ = progress.send(event.clone()).await;
        }
        events.push(event);
    }
    events
}

pub(crate) async fn drain_events_until<S>(
    mut stream: S,
    progress: Option<EventProgress>,
    wait: Duration,
) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    let deadline = sleep(wait);
    tokio::pin!(deadline);
    let mut events = Vec::new();
    loop {
        tokio::select! {
            _ = &mut deadline => return events,
            event = stream.next() => match event {
                Some(event) => {
                    if let Some(progress) = &progress {
                        let _ = progress.send(event.clone()).await;
                    }
                    events.push(event);
                }
                None => return events,
            }
        }
    }
}
