//! Exact nostr-sdk relay reads with owner-supplied admission settlement.

use super::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo, RelayReadResult};
use anyhow::{bail, Context};
use log::warn;
use nostr_sdk::pool::RelayNotification;
use nostr_sdk::{Client, RelayStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinSet;
use tokio::time::timeout;

const RELAY_READINESS_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct SdkRelayIo {
    client: Arc<Client>,
    readiness_timeout: Duration,
}

impl SdkRelayIo {
    pub(crate) fn new(client: Arc<Client>) -> Self {
        Self::with_components(client, RELAY_READINESS_TIMEOUT)
    }

    #[cfg(test)]
    pub(crate) fn with_readiness_timeout(client: Arc<Client>, timeout: Duration) -> Self {
        Self::with_components(client, timeout)
    }

    fn with_components(client: Arc<Client>, readiness_timeout: Duration) -> Self {
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
    fn read(&self, mut request: RelayReadIo) -> RelayIoFuture<'_, RelayReadResult> {
        Box::pin(async move {
            let mut admissions = request.admissions.take();
            let outcome = crate::relay::scoped_read::read(
                self.client.clone(),
                request,
                self.readiness_timeout,
            )
            .await?;
            if let Some(batch) = &mut admissions {
                batch.settle(&outcome.completed_relays, &outcome.failed_relays);
            }
            outcome.result.context("relay query failed")
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
