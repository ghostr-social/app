//! Linearized session-safe access to the process-wide Nostr relay pool.

use crate::plan_executor::PlanFailure;
use crate::relay_io::{RelayIo, SdkRelayIo};
use crate::relay_pool_roles::RelayPoolRoles;
use crate::relay_removal::RelayRoleIo;
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use nostr_sdk::{Client, Event, PublicKey};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{watch, Mutex, OwnedRwLockReadGuard, RwLock};

pub use crate::relay_pool_roles::RelayPoolConfiguration;
pub use crate::relay_pool_transition::RelayPoolTransition;

const ACCOUNT_MISMATCH_MESSAGE: &str = "the signed event does not belong to the active account";
const REQUEST_ACCOUNT_MISMATCH_MESSAGE: &str =
    "the feed request does not belong to the active account";
const REQUEST_SESSION_MISMATCH_MESSAGE: &str = "the feed request belongs to a stale Nostr session";

#[derive(Clone)]
pub struct RelayPoolOwner {
    pub(super) roles: Arc<RelayPoolRoles>,
    pub(super) barrier: Arc<RwLock<()>>,
    pub(super) transition_serial: Arc<Mutex<()>>,
    pub(super) lifecycle: Arc<StdMutex<Lifecycle>>,
    pub(super) cancellations: watch::Sender<u64>,
    pub(super) io: Arc<dyn RelayIo>,
}

pub struct RelayReadRequest {
    pub session: SessionGeneration,
    pub relays: Option<Vec<String>>,
    pub query: crate::search_queries::PlannedQuery,
    pub progress: Option<crate::plan_executor::EventProgress>,
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
        let io = Arc::new(SdkRelayIo::new(client.clone()));
        Self::with_io(client, configuration, io)
    }

    pub fn with_io(
        client: Arc<Client>,
        configuration: RelayPoolConfiguration,
        io: Arc<dyn RelayIo>,
    ) -> Self {
        let (cancellations, _) = watch::channel(0);
        Self {
            roles: Arc::new(RelayPoolRoles::new(RelayRoleIo::sdk(client), configuration)),
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
        }
    }

    #[cfg(test)]
    pub(crate) async fn read(&self, request: RelayReadRequest) -> Result<Vec<Event>, PlanFailure> {
        self.begin_route(request.session).await?.read(request).await
    }

    #[cfg(test)]
    pub(crate) async fn broadcast(&self, request: RelayBroadcastRequest) -> anyhow::Result<()> {
        let route = self
            .begin_route(request.session)
            .await
            .map_err(|failure| anyhow::anyhow!(failure.message))?;
        route.broadcast(request).await
    }

    pub async fn begin_account_request(
        &self,
        expected_account: Option<PublicKey>,
        expected_session: SessionGeneration,
    ) -> anyhow::Result<OwnedRwLockReadGuard<()>> {
        let barrier = self.barrier.clone().read_owned().await;
        let active_session = self.active_session(expected_account)?;
        anyhow::ensure!(
            active_session == expected_session,
            REQUEST_SESSION_MISMATCH_MESSAGE
        );
        Ok(barrier)
    }

    pub async fn account_session(
        &self,
        expected_account: Option<PublicKey>,
    ) -> anyhow::Result<SessionGeneration> {
        let _barrier = self.barrier.clone().read_owned().await;
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

pub(super) fn locked(lifecycle: &StdMutex<Lifecycle>) -> std::sync::MutexGuard<'_, Lifecycle> {
    lifecycle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn session_failure() -> PlanFailure {
    PlanFailure::new(SESSION_RESET_MESSAGE)
}
