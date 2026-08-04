//! Social writes over the FFI (plan §5.5): Dart signs, Rust validates
//! and broadcasts. Keys never cross this boundary — only pre-signed
//! event JSON, and only when its id and signature verify.

use crate::api::feed_runtime::DiscoveryRuntime;
use crate::api::runtime_registry;
use crate::discovery::outbox_directory::{max_outbox_relays, OutboxDirectory};
use crate::engine::DataUsageLevel;
use anyhow::{anyhow, Context};
use flutter_rust_bridge::frb;
use nostr_sdk::{Client, Event, JsonUtil, PublicKey};

/// Validates one pre-signed event and publishes it with outbox-aware
/// relay selection: the author's declared write relays after the
/// bootstrap set, capped by the current data-usage level.
#[frb]
pub async fn ffi_broadcast_event(signed_event_json: String) -> anyhow::Result<()> {
    let event = verified_event(&signed_event_json)?;
    let engine = runtime_registry::engine()?;
    let level = engine.tracked.level();
    engine.discovery.broadcast(event, level).await
}

/// The parsed event, if and only if its id and signature verify.
pub(crate) fn verified_event(json: &str) -> anyhow::Result<Event> {
    let event = Event::from_json(json).map_err(|error| anyhow!("unparseable event JSON: {error}"))?;
    event
        .verify()
        .map_err(|_| anyhow!("the event id or signature does not verify"))?;
    Ok(event)
}

/// Where one author's events go: bootstrap relays first, then their
/// declared write relays, capped like every outbox lookup.
pub(crate) fn broadcast_relays(
    outbox: &OutboxDirectory,
    author: &PublicKey,
    level: DataUsageLevel,
) -> Vec<String> {
    outbox.relays_for_authors(&[*author], max_outbox_relays(level))
}

impl DiscoveryRuntime {
    pub(crate) async fn broadcast(&self, event: Event, level: DataUsageLevel) -> anyhow::Result<()> {
        let relays = broadcast_relays(&*self.outbox.read().await, &event.pubkey, level);
        if relays.is_empty() {
            self.client.send_event(event).await.context("broadcast failed")?;
            return Ok(());
        }
        ensure_relays(&self.client, &relays).await;
        self.client
            .send_event_to(relays, event)
            .await
            .context("broadcast failed")?;
        Ok(())
    }
}

/// Explicit relays are ensured in the pool and connected before the
/// send, like the plan executor does before a fetch.
async fn ensure_relays(client: &Client, urls: &[String]) {
    for url in urls {
        if client.add_relay(url).await.unwrap_or(false) {
            let _ = client.connect_relay(url).await;
        }
    }
}
