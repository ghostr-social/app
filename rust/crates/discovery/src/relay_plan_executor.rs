//! Resolves relay sets and fans the plan out concurrently through one
//! session-safe relay-pool owner.

use crate::event_cache::EventCache;
use crate::live_search_relays::LiveSearchRelays;
use crate::outbox_directory::{max_outbox_relays, OutboxDirectory};
use crate::plan_executor::{PlanExecutor, PlanFuture, PlanPageFuture, PlannedRetrieval};
#[cfg(test)]
use crate::relay_plan_routes::outbox_relays as resolved_outbox;
use crate::relay_plan_routes::plan_outbox_relays as resolved_plan_outboxes;
#[cfg(test)]
use crate::relay_pool_owner::RelayPoolConfiguration;
use crate::relay_pool_owner::RelayPoolOwner;
use crate::retrieval_types::{EventProgress, PlanFailure};
#[cfg(test)]
use crate::search_queries::OutboxLookup;
use crate::search_queries::QueryPlan;
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use ghostr_engine::DataUsageLevel;
#[cfg(test)]
use nostr_sdk::Client;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

mod execution;
mod fetches;
mod profile_enrichment;

/// Shared live outbox directory; ingestion happens on the subscription side.
pub type SharedOutboxDirectory = Arc<RwLock<OutboxDirectory>>;

#[derive(Clone)]
pub struct RelayPlanExecutor {
    relay_pool: Arc<RelayPoolOwner>,
    /// The private session event pool shared with every clone, read back
    /// so a repeat query serves stored UNION network.
    cache: Arc<EventCache>,
    search_relays: LiveSearchRelays,
    outbox: SharedOutboxDirectory,
    /// Shared with every clone so a live data-usage change reaches the
    /// executor the scheduler already holds.
    outbox_cap: Arc<AtomicUsize>,
}

impl RelayPlanExecutor {
    #[cfg(test)]
    pub fn new(
        client: Arc<Client>,
        search_relays: Vec<String>,
        outbox: SharedOutboxDirectory,
        level: DataUsageLevel,
    ) -> Self {
        let configuration = RelayPoolConfiguration {
            read_relays: Vec::new(),
            search_relays: search_relays.clone(),
        };
        let relay_pool = Arc::new(RelayPoolOwner::new(client, configuration));
        Self::with_owner(relay_pool, search_relays, outbox, level)
    }

    pub fn with_owner(
        relay_pool: Arc<RelayPoolOwner>,
        search_relays: Vec<String>,
        outbox: SharedOutboxDirectory,
        level: DataUsageLevel,
    ) -> Self {
        Self {
            cache: Arc::new(EventCache::session()),
            relay_pool,
            search_relays: LiveSearchRelays::new(search_relays),
            outbox,
            outbox_cap: Arc::new(AtomicUsize::new(max_outbox_relays(level))),
        }
    }

    /// The session event pool this executor reads and files into.
    pub fn cache(&self) -> Arc<EventCache> {
        self.cache.clone()
    }

    /// Scopes the pool to whoever this plan's feed named, before any of
    /// its queries reads it; reports whether the pool was emptied.
    #[cfg(test)]
    pub(crate) async fn adopt_plan_viewer(&self, plan: &QueryPlan) -> bool {
        self.cache.adopt(plan.viewer).await
    }

    /// Live outbox fan-out change (`ffi_set_delivery_config`): the next
    /// query of every open feed uses the new cap.
    pub fn set_data_usage(&self, level: DataUsageLevel) {
        self.outbox_cap
            .store(max_outbox_relays(level), Ordering::Relaxed);
    }

    pub fn set_search_relays(&self, relays: Vec<String>) {
        self.search_relays.replace(relays);
    }

    pub fn search_relays(&self) -> Vec<String> {
        self.search_relays.snapshot()
    }

    async fn adopt_session_viewer(
        &self,
        session: SessionGeneration,
        plan: &QueryPlan,
    ) -> Result<(), PlanFailure> {
        self.cache
            .adopt_for(session, plan.viewer)
            .await
            .map(|_| ())
            .ok_or_else(|| PlanFailure::new(SESSION_RESET_MESSAGE))
    }

    /// Resolves author write relays or the viewer's ranked discovery
    /// relays. An empty result falls back to the bootstrap pool.
    #[cfg(test)]
    pub(crate) async fn outbox_relays(&self, lookup: &OutboxLookup) -> Option<Vec<String>> {
        let cap = self.outbox_cap.load(Ordering::Relaxed);
        let directory = self.outbox.read().await;
        resolved_outbox(&directory, lookup, cap)
    }

    #[cfg(test)]
    pub(crate) async fn plan_outbox_relays(&self, plan: &QueryPlan) -> Vec<Option<Vec<String>>> {
        let cap = self.outbox_cap.load(Ordering::Relaxed);
        let directory = self.outbox.read().await;
        resolved_plan_outboxes(&directory, plan, cap)
    }

    pub(crate) async fn session_plan_outboxes(
        &self,
        session: SessionGeneration,
        plan: &QueryPlan,
    ) -> Result<Vec<Option<Vec<String>>>, PlanFailure> {
        let cap = self.outbox_cap.load(Ordering::Relaxed);
        let directory = self.outbox.read().await;
        if !directory.is_session(session) {
            return Err(PlanFailure::new(SESSION_RESET_MESSAGE));
        }
        Ok(resolved_plan_outboxes(&directory, plan, cap))
    }
}

impl PlanExecutor for RelayPlanExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        Box::pin(self.clone().run(retrieval, None))
    }

    fn execute_page_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanPageFuture {
        Box::pin(self.clone().run_page(retrieval, Some(progress)))
    }
}
