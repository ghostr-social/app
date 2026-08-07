//! Runtime bridge from typed generic queries to the discovery scheduler.

use crate::api::runtime::discovery::{lock, DiscoveryRuntime};
use crate::discovery::query::events::plan_event_queries;
use crate::discovery::retrieval_types::PlanFailure;
use nostr_sdk::{Event, Filter};

impl DiscoveryRuntime {
    pub(crate) async fn query_events(
        &self,
        filters: Vec<Filter>,
    ) -> Result<Vec<Event>, PlanFailure> {
        let session = lock(&self.state).session_generation();
        self.handle
            .query(session, plan_event_queries(filters))
            .await
    }
}
