//! Resolves relay sets and fans the plan out concurrently through one
//! session-safe relay-pool owner.

use crate::cache::EventCache;

use crate::execution::routes::plan_outbox_relays as resolved_plan_outboxes;
use crate::outbox::directory::{max_outbox_relays, SharedOutboxDirectory};
use crate::plan_executor::{PlanExecutor, PlanFuture, PlanPageFuture, PlannedRetrieval};
use crate::query::live_search_relays::LiveSearchRelays;

use crate::query::search::QueryPlan;

use crate::relay::pool::RelayPoolOwner;
use crate::retrieval_types::{EventProgress, PlanFailure};
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use core::sync::atomic::{AtomicUsize, Ordering};
use ghostr_engine::DataUsageLevel;

use std::sync::Arc;

pub(crate) mod deletion_enrichment;
mod deletion_hints;
mod deletion_planning;
mod deletion_targets;
mod execution;
mod fetches;
pub(crate) mod profile_enrichment;
mod repost_retry;
mod repost_support;
mod target_dependencies;
pub(crate) mod target_enrichment;
mod target_hints;
mod target_planning;

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
        std::sync::Arc::clone(&self.cache)
    }

    pub fn with_cache(mut self, cache: Arc<EventCache>) -> Self {
        self.cache = cache;
        self
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

    fn execute_page(&self, retrieval: PlannedRetrieval) -> PlanPageFuture {
        Box::pin(self.clone().run_page(retrieval, None))
    }

    fn execute_page_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanPageFuture {
        Box::pin(self.clone().run_page(retrieval, Some(progress)))
    }
}

#[cfg(test)]
#[path = "relay_executor_axiom_test.rs"]
pub(crate) mod axiom_test_support;
