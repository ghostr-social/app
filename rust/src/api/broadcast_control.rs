//! Social writes over the FFI (plan §5.5): Dart signs, Rust validates
//! and broadcasts. Keys never cross this boundary — only pre-signed
//! event JSON, and only when its id and signature verify.

use crate::api::runtime::discovery::DiscoveryRuntime;
use crate::api::runtime::registry;
use crate::discovery::outbox::directory::{max_outbox_relays, OutboxDirectory};
use crate::discovery::relay::pool::RelayBroadcastRequest;
use crate::discovery::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use crate::engine::DataUsageLevel;
use anyhow::anyhow;
use flutter_rust_bridge::frb;
use nostr_sdk::{Event, JsonUtil, PublicKey};

/// Validates one pre-signed event and publishes it with outbox-aware
/// relay selection: the author's declared write relays after the
/// bootstrap set, capped by the current data-usage level.
#[frb]
pub async fn ffi_broadcast_event(signed_event_json: String) -> anyhow::Result<()> {
    let event = verified_event(&signed_event_json)?;
    let engine = registry::engine()?;
    let session = engine.discovery.session_generation();
    let level = engine.tracked.level();
    engine
        .discovery
        .broadcast(session, event.clone(), level)
        .await?;
    engine.discovery.remember_accepted(session, &event).await;
    Ok(())
}

/// The parsed event, if and only if its id and signature verify.
pub(crate) fn verified_event(json: &str) -> anyhow::Result<Event> {
    let event =
        Event::from_json(json).map_err(|error| anyhow!("unparseable event JSON: {error}"))?;
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
    pub(crate) async fn broadcast(
        &self,
        session: SessionGeneration,
        event: Event,
        level: DataUsageLevel,
    ) -> anyhow::Result<()> {
        let route = self
            .relay_pool
            .begin_route(session)
            .await
            .map_err(|failure| anyhow!(failure.message))?;
        let directory = self.outbox.read().await;
        anyhow::ensure!(directory.is_session(session), SESSION_RESET_MESSAGE);
        let relays = broadcast_relays(&directory, &event.pubkey, level);
        drop(directory);
        route
            .broadcast(RelayBroadcastRequest {
                session,
                relays,
                event,
            })
            .await
    }
}
