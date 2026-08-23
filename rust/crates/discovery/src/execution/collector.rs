//! Joins every routed fetch so no relay lease outlives its retrieval.

use crate::execution::fetch::FetchedEvents;
use crate::feed::cursor::wire_retrieval_cursor;
use crate::query::search::QueryRole;
use crate::retrieval_types::PlanFailure;
use crate::session_generation::SESSION_RESET_MESSAGE;
use nostr_sdk::{Event, Timestamp};
use tokio::task::JoinHandle;

pub(crate) type FetchHandle = JoinHandle<Result<FetchedEvents, PlanFailure>>;

pub(crate) struct CollectedPage {
    pub events: Vec<Event>,
    pub cursor: Option<Timestamp>,
    pub complete: bool,
}

pub(crate) struct CollectedEvents {
    pub events: Vec<Event>,
    pub complete: bool,
}

struct AbortFetch(FetchHandle);

#[derive(Default)]
struct PageCollection {
    pages: Vec<FetchedEvents>,
    primary_failure: Option<PlanFailure>,
    additive_failure: Option<PlanFailure>,
    reset_failure: Option<PlanFailure>,
}

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
    let fetches = fetches
        .into_iter()
        .map(|(role, fetch)| (role, AbortFetch(fetch)))
        .collect::<Vec<_>>();
    let mut collection = PageCollection::default();
    for (role, fetch) in fetches {
        collection.record(role, joined(fetch).await);
    }
    collection.finish()
}

impl PageCollection {
    fn record(&mut self, role: QueryRole, result: Result<FetchedEvents, PlanFailure>) {
        let failure = match result {
            Ok(fetched) => return self.pages.push(fetched),
            Err(failure) => failure,
        };
        if failure.message == SESSION_RESET_MESSAGE {
            self.reset_failure.get_or_insert(failure);
            return;
        }
        match role {
            QueryRole::Primary => self.primary_failure.get_or_insert(failure),
            QueryRole::Additive => self.additive_failure.get_or_insert(failure),
        };
    }

    fn finish(self) -> Result<CollectedPage, PlanFailure> {
        let Self {
            pages,
            primary_failure,
            additive_failure,
            reset_failure,
        } = self;
        if let Some(failure) = reset_failure {
            return Err(failure);
        }
        let failure = primary_failure.or(additive_failure);
        if pages.is_empty() {
            if let Some(failure) = failure {
                return Err(failure);
            }
        }
        let complete = failure.is_none() && pages.iter().all(|page| page.wire_complete);
        Ok(Self::page(pages, complete))
    }

    fn page(pages: Vec<FetchedEvents>, complete: bool) -> CollectedPage {
        let cursor = complete
            .then(|| wire_retrieval_cursor(pages.iter().filter_map(|page| page.fresh_boundary)))
            .flatten();
        CollectedPage {
            events: pages.into_iter().flat_map(|page| page.events).collect(),
            cursor,
            complete,
        }
    }
}

async fn joined(mut fetch: AbortFetch) -> Result<FetchedEvents, PlanFailure> {
    (&mut fetch.0)
        .await
        .unwrap_or_else(|error| Err(PlanFailure::new(error.to_string())))
}
