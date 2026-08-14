//! One relay filter fetch, including warm-cache fallback.

use crate::cache::EventCache;
use crate::execution::cache_fallback::cached_or_failure;
use crate::feed::cursor::wire_page_boundary;
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

pub(crate) struct FetchedEvents {
    pub events: Vec<Event>,
    pub fresh_boundary: Option<nostr_sdk::Timestamp>,
    pub wire_complete: bool,
}

impl FetchedEvents {
    #[cfg(test)]
    pub(crate) fn fresh(events: Vec<Event>) -> Self {
        let fresh_boundary = wire_page_boundary(&events);
        Self {
            events,
            fresh_boundary,
            wire_complete: true,
        }
    }

    fn cached(events: Vec<Event>) -> Self {
        Self {
            events,
            fresh_boundary: None,
            wire_complete: false,
        }
    }
}

pub(crate) async fn fetch(request: RelayFetch) -> Result<FetchedEvents, PlanFailure> {
    let filter = request.query.filter.clone();
    publish_cached(&request, &filter).await;
    let progressive = request.progress.is_some();
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
            if progressive {
                return Err(error);
            }
            let events =
                cached_or_failure(&request.cache, request.session, &filter, error.message).await?;
            return Ok(FetchedEvents::cached(events));
        }
    };
    let fresh_boundary = wire_page_boundary(&fetched);
    let events = request
        .cache
        .union_for(request.session, &filter, fetched)
        .await
        .ok_or_else(|| PlanFailure::new(SESSION_RESET_MESSAGE))?;
    Ok(FetchedEvents {
        events,
        fresh_boundary,
        wire_complete: true,
    })
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
