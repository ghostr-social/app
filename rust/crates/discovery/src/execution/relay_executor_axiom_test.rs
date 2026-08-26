use super::*;

use crate::execution::routes::outbox_relays as resolved_outbox;

use crate::query::search::OutboxLookup;

use crate::relay::pool::RelayPoolConfiguration;

use nostr_sdk::Client;

impl RelayPlanExecutor {
    pub(crate) fn new(
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
    /// Scopes the pool to whoever this plan's feed named, before any of
    /// its queries reads it; reports whether the pool was emptied.
    pub(crate) async fn adopt_plan_viewer(&self, plan: &QueryPlan) -> bool {
        self.cache.adopt(plan.viewer).await
    }
    /// Resolves author write relays or the viewer's ranked discovery
    /// relays. An empty result falls back to the bootstrap pool.
    pub(crate) async fn outbox_relays(&self, lookup: &OutboxLookup) -> Option<Vec<String>> {
        let cap = self.outbox_cap.load(Ordering::Relaxed);
        let directory = self.outbox.read().await;
        resolved_outbox(&directory, lookup, cap)
    }
    pub(crate) async fn plan_outbox_relays(&self, plan: &QueryPlan) -> Vec<Option<Vec<String>>> {
        let cap = self.outbox_cap.load(Ordering::Relaxed);
        let directory = self.outbox.read().await;
        resolved_plan_outboxes(&directory, plan, cap)
    }
}
