//! Linearized session-safe access to the process-wide Nostr relay pool.

use crate::relay::health::RelayHealth;
use crate::relay::io::{RelayIo, SdkRelayIo};
use crate::relay::removal::RelayRoleIo;
use crate::relay::roles::RelayPoolRoles;
use crate::retrieval_types::PlanFailure;
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use nostr_sdk::{Client, Event, PublicKey};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{watch, Mutex, OwnedRwLockReadGuard, RwLock};

pub use crate::relay::roles::RelayPoolConfiguration;
pub use crate::relay::transition::RelayPoolTransition;

const ACCOUNT_MISMATCH_MESSAGE: &str = "the signed event does not belong to the active account";
const REQUEST_ACCOUNT_MISMATCH_MESSAGE: &str =
    "the feed request does not belong to the active account";
const REQUEST_SESSION_MISMATCH_MESSAGE: &str = "the feed request belongs to a stale Nostr session";

#[derive(Clone)]
pub struct RelayPoolOwner {
    pub(super) roles: Arc<RelayPoolRoles>,
    pub(super) barrier: Arc<RwLock<()>>,
    pub(crate) transition_serial: Arc<Mutex<()>>,
    pub(super) lifecycle: Arc<StdMutex<Lifecycle>>,
    pub(super) cancellations: watch::Sender<u64>,
    pub(super) io: Arc<dyn RelayIo>,
    pub(super) health: Arc<RelayHealth>,
}

pub(crate) struct RelayReadRequest {
    pub(crate) session: SessionGeneration,
    pub(crate) relays: Option<Vec<String>>,
    pub(crate) query: crate::query::search::PlannedQuery,
    pub(crate) progress: Option<crate::retrieval_types::EventProgress>,
}

pub struct RelayBroadcastRequest {
    pub session: SessionGeneration,
    pub relays: Vec<String>,
    pub event: Event,
}

pub(super) struct Lifecycle {
    pub session: SessionGeneration,
    pub expected_account: Option<PublicKey>,
    pub epoch: u64,
    pub pending_resets: usize,
}

impl RelayPoolOwner {
    pub fn new(client: Arc<Client>, configuration: RelayPoolConfiguration) -> Self {
        let health = Arc::new(RelayHealth::new());
        let io = Arc::new(SdkRelayIo::new(std::sync::Arc::clone(&client)));
        Self::with_components(configuration, io, RelayRoleIo::sdk(client), health)
    }

    fn with_components(
        configuration: RelayPoolConfiguration,
        io: Arc<dyn RelayIo>,
        roles: RelayRoleIo,
        health: Arc<RelayHealth>,
    ) -> Self {
        let (cancellations, _) = watch::channel(0);
        Self {
            roles: Arc::new(RelayPoolRoles::new(roles, configuration)),
            barrier: Arc::new(RwLock::new(())),
            transition_serial: Arc::new(Mutex::new(())),
            lifecycle: Arc::new(StdMutex::new(Lifecycle {
                session: SessionGeneration::initial(),
                expected_account: None,
                epoch: 0,
                pending_resets: 0,
            })),
            cancellations,
            io,
            health,
        }
    }

    /// Begins an account-scoped request while holding the session barrier.
    ///
    /// # Errors
    ///
    /// Returns an error when the expected account or session is no longer active.
    pub async fn begin_account_request(
        &self,
        expected_account: Option<PublicKey>,
        expected_session: SessionGeneration,
    ) -> anyhow::Result<OwnedRwLockReadGuard<()>> {
        let barrier = std::sync::Arc::clone(&self.barrier).read_owned().await;
        let active_session = self.active_session(expected_account)?;
        anyhow::ensure!(
            active_session == expected_session,
            REQUEST_SESSION_MISMATCH_MESSAGE
        );
        Ok(barrier)
    }

    /// Returns the generation for the expected active account.
    ///
    /// # Errors
    ///
    /// Returns an error when the expected account is not active.
    pub async fn account_session(
        &self,
        expected_account: Option<PublicKey>,
    ) -> anyhow::Result<SessionGeneration> {
        let _barrier = std::sync::Arc::clone(&self.barrier).read_owned().await;
        self.active_session(expected_account)
    }

    pub(super) fn ensure_session(
        &self,
        session: SessionGeneration,
        epoch: u64,
    ) -> Result<(), PlanFailure> {
        let lifecycle = locked(&self.lifecycle);
        if lifecycle.pending_resets > 0 || lifecycle.session != session || lifecycle.epoch != epoch
        {
            return Err(session_failure());
        }
        Ok(())
    }

    pub(super) fn ensure_author(&self, author: &PublicKey) -> anyhow::Result<()> {
        let lifecycle = locked(&self.lifecycle);
        anyhow::ensure!(
            lifecycle.expected_account.as_ref() == Some(author),
            ACCOUNT_MISMATCH_MESSAGE
        );
        Ok(())
    }

    fn active_session(
        &self,
        expected_account: Option<PublicKey>,
    ) -> anyhow::Result<SessionGeneration> {
        let lifecycle = locked(&self.lifecycle);
        anyhow::ensure!(
            lifecycle.pending_resets == 0 && lifecycle.expected_account == expected_account,
            REQUEST_ACCOUNT_MISMATCH_MESSAGE
        );
        Ok(lifecycle.session)
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "pool/test_support.rs"]
mod test_support;

pub(super) fn locked(lifecycle: &StdMutex<Lifecycle>) -> std::sync::MutexGuard<'_, Lifecycle> {
    lifecycle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn session_failure() -> PlanFailure {
    PlanFailure::new(SESSION_RESET_MESSAGE)
}

#[cfg(test)]
#[path = "pool_axiom_test.rs"]
mod axiom_test_support;
