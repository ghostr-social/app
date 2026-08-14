//! Joins every routed fetch so no relay lease outlives its retrieval.

use crate::execution::fetch::FetchedEvents;
use crate::feed::cursor::wire_retrieval_cursor;
use crate::query::search::QueryRole;
use crate::retrieval_types::PlanFailure;
use nostr_sdk::{Event, Timestamp};
use tokio::task::JoinHandle;

pub(crate) type FetchHandle = JoinHandle<Result<FetchedEvents, PlanFailure>>;

pub(crate) struct CollectedPage {
    pub events: Vec<Event>,
    pub cursor: Option<Timestamp>,
}

pub(crate) struct CollectedEvents {
    pub events: Vec<Event>,
    pub complete: bool,
}

struct AbortFetch(FetchHandle);

impl Drop for AbortFetch {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[cfg(test)]
pub(crate) async fn collect_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Result<Vec<Event>, PlanFailure> {
    collect_page(fetches).await.map(|page| page.events)
}

pub(crate) async fn collect_best_effort_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Vec<Event> {
    collect_partial_events(fetches).await.events
}

pub(crate) async fn collect_partial_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> CollectedEvents {
    let outcomes = collect_partial_fetches(fetches).await;
    let complete = outcomes.iter().all(|outcome| outcome.complete);
    let events = outcomes
        .into_iter()
        .flat_map(|outcome| outcome.events)
        .collect();
    CollectedEvents { events, complete }
}

pub(crate) async fn collect_partial_fetches(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Vec<CollectedEvents> {
    let fetches = fetches
        .into_iter()
        .map(|(_, fetch)| AbortFetch(fetch))
        .collect::<Vec<_>>();
    let mut outcomes = Vec::with_capacity(fetches.len());
    for fetch in fetches {
        let outcome = joined(fetch).await.map_or_else(
            |_| CollectedEvents {
                events: Vec::new(),
                complete: false,
            },
            |fetched| CollectedEvents {
                events: fetched.events,
                complete: fetched.wire_complete,
            },
        );
        outcomes.push(outcome);
    }
    outcomes
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
    let cursor = wire_retrieval_cursor(pages.iter().filter_map(|page| page.fresh_boundary));
    Ok(CollectedPage {
        events: pages.into_iter().flat_map(|page| page.events).collect(),
        cursor,
    })
}

async fn joined(mut fetch: AbortFetch) -> Result<FetchedEvents, PlanFailure> {
    (&mut fetch.0)
        .await
        .unwrap_or_else(|error| Err(PlanFailure::new(error.to_string())))
}
