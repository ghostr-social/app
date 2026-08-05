//! One relay filter fetch, including warm-cache fallback.

use crate::discovery::cache_fallback::cached_or_failure;
use crate::discovery::event_cache::EventCache;
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::relay_pool_owner::RelayReadRequest;
use crate::discovery::relay_pool_route::RelayPoolRoute;
use crate::discovery::search_queries::PlannedQuery;
use crate::discovery::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use nostr_sdk::Event;
use std::sync::Arc;

pub(crate) struct RelayFetch {
    pub route: Arc<RelayPoolRoute>,
    pub cache: Arc<EventCache>,
    pub session: SessionGeneration,
    pub relays: Option<Vec<String>>,
    pub query: PlannedQuery,
}

pub(crate) async fn fetch(request: RelayFetch) -> Result<Vec<Event>, PlanFailure> {
    let filter = request.query.filter.clone();
    let fetched = request
        .route
        .read(RelayReadRequest {
            session: request.session,
            relays: request.relays,
            query: request.query,
        })
        .await;
    let fetched = match fetched {
        Ok(events) => events,
        Err(error) => {
            return cached_or_failure(&request.cache, request.session, &filter, error.message)
                .await;
        }
    };
    request
        .cache
        .union_for(request.session, &filter, fetched)
        .await
        .ok_or_else(|| PlanFailure::new(SESSION_RESET_MESSAGE))
}
