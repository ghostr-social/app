//! Nostr account-session lifecycle over the installed engine.

use crate::api::runtime::discovery::{lock, DiscoveryRuntime};
use crate::api::runtime::registry;
use flutter_rust_bridge::frb;
use nostr_sdk::PublicKey;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Typed failures from [`ffi_reset_nostr_session`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NostrSessionResetError {
    EngineNotStarted,
    InvalidExpectedPublicKey,
}

impl Display for NostrSessionResetError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EngineNotStarted => formatter.write_str("the Nostr engine is not started"),
            Self::InvalidExpectedPublicKey => {
                formatter.write_str("the expected Nostr public key is invalid")
            }
        }
    }
}

impl Error for NostrSessionResetError {}

/// Clears account-scoped Nostr state without stopping the media engine.
#[frb]
pub async fn ffi_reset_nostr_session(
    expected_public_key_hex: Option<String>,
) -> Result<(), NostrSessionResetError> {
    let expected_account = expected_public_key_hex
        .map(|value| PublicKey::from_hex(&value))
        .transpose()
        .map_err(|_| NostrSessionResetError::InvalidExpectedPublicKey)?;
    let engine = registry::engine_if_running().ok_or(NostrSessionResetError::EngineNotStarted)?;
    engine.discovery.reset_session(expected_account).await;
    Ok(())
}

impl DiscoveryRuntime {
    pub(crate) fn session_generation(
        &self,
    ) -> crate::discovery::session_generation::SessionGeneration {
        lock(&self.state).session_generation()
    }

    pub(crate) async fn reset_session(&self, expected_account: Option<PublicKey>) {
        let mut transition = self.relay_pool.begin_reset().await;
        let generation = lock(&self.state).reset_session();
        let _ = self.handle.reset_session().await;
        self.bootstrap.reset_session(generation);
        self.outbox.write().await.reset_session(generation);
        self.executor.cache().reset_session(generation).await;
        let _ = self.client.database().wipe().await;
        transition.reset_session(generation, expected_account).await;
    }
}
