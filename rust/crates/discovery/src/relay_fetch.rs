//! One relay filter fetch, including warm-cache fallback.

use crate::cache_fallback::cached_or_failure;
use crate::event_cache::EventCache;
use crate::plan_executor::{EventProgress, PlanFailure};
use crate::relay_pool_owner::RelayReadRequest;
use crate::relay_pool_route::RelayPoolRoute;
use crate::search_queries::PlannedQuery;
use crate::session_generation::{SessionGeneration, SESSION_RESET_MESSAGE};
use nostr_sdk::Event;
use std::sync::Arc;

pub(crate) struct RelayFetch {
    pub route: Arc<RelayPoolRoute>,
    pub cache: Arc<EventCache>,
    pub session: SessionGeneration,
    pub relays: Option<Vec<String>>,
    pub query: PlannedQuery,
    pub progress: Option<EventProgress>,
}

pub(crate) async fn fetch(request: RelayFetch) -> Result<Vec<Event>, PlanFailure> {
    let filter = request.query.filter.clone();
    publish_cached(&request, &filter).await;
    let fetched = request
        .route
        .read(RelayReadRequest {
            session: request.session,
            relays: request.relays,
            query: request.query,
            progress: request.progress,
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

async fn publish_cached(request: &RelayFetch, filter: &nostr_sdk::Filter) {
    let Some(progress) = &request.progress else {
        return;
    };
    let Some(events) = request.cache.stored_for(request.session, filter).await else {
        return;
    };
    for event in events {
        let _ = progress.send(event).await;
    }
}
