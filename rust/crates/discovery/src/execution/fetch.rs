//! One relay filter fetch, including warm-cache fallback.

use crate::cache::EventCache;
use crate::execution::cache_fallback::cached_or_failure;
use crate::query::search::PlannedQuery;
use crate::relay::pool::RelayReadRequest;
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::{EventProgress, PlanFailure};
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
