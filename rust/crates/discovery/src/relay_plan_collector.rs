//! Joins every routed fetch so no relay lease outlives its retrieval.

use crate::feed_cursor::wire_retrieval_cursor;
use crate::retrieval_types::PlanFailure;
use crate::search_queries::QueryRole;
use nostr_sdk::{Event, Timestamp};
use tokio::task::JoinHandle;

pub(crate) type FetchHandle = JoinHandle<Result<Vec<Event>, PlanFailure>>;

pub(crate) struct CollectedPage {
    pub events: Vec<Event>,
    pub cursor: Option<Timestamp>,
}

struct AbortFetch(FetchHandle);

impl Drop for AbortFetch {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(crate) async fn collect_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Result<Vec<Event>, PlanFailure> {
    collect_page(fetches).await.map(|page| page.events)
}

pub(crate) async fn collect_page(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Result<CollectedPage, PlanFailure> {
    let mut pages = Vec::new();
    let mut primary_failure = None;
    let mut additive_failure = None;
    let fetches = fetches
        .into_iter()
        .map(|(role, fetch)| (role, AbortFetch(fetch)))
        .collect::<Vec<_>>();
    for (role, fetch) in fetches {
        match (role, joined(fetch).await) {
            (_, Ok(fetched)) => pages.push(fetched),
            (QueryRole::Primary, Err(failure)) => primary_failure = Some(failure),
            (QueryRole::Additive, Err(failure)) => additive_failure = Some(failure),
        }
    }
    if let Some(failure) = primary_failure.or(additive_failure) {
        return Err(failure);
    }
    let cursor = wire_retrieval_cursor(pages.iter().map(Vec::as_slice));
    Ok(CollectedPage {
        events: pages.into_iter().flatten().collect(),
        cursor,
    })
}

async fn joined(mut fetch: AbortFetch) -> Result<Vec<Event>, PlanFailure> {
    (&mut fetch.0)
        .await
        .unwrap_or_else(|error| Err(PlanFailure::new(error.to_string())))
}
