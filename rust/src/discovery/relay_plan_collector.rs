//! Joins every routed fetch so no relay lease outlives its retrieval.

use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::search_queries::QueryRole;
use log::warn;
use nostr_sdk::Event;
use tokio::task::JoinHandle;

pub(crate) type FetchHandle = JoinHandle<Result<Vec<Event>, PlanFailure>>;

pub(crate) async fn collect_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Result<Vec<Event>, PlanFailure> {
    let mut events = Vec::new();
    let mut primary_failure = None;
    for (role, fetch) in fetches {
        match (role, joined(fetch).await) {
            (_, Ok(fetched)) => events.extend(fetched),
            (QueryRole::Primary, Err(failure)) => primary_failure = Some(failure),
            (QueryRole::Additive, Err(failure)) => warn_additive(failure),
        }
    }
    primary_failure.map_or(Ok(events), Err)
}

fn warn_additive(failure: PlanFailure) {
    warn!(
        "Skipping a failed additive discovery query: {}",
        failure.message
    );
}

async fn joined(fetch: FetchHandle) -> Result<Vec<Event>, PlanFailure> {
    fetch
        .await
        .unwrap_or_else(|error| Err(PlanFailure::new(error.to_string())))
}
