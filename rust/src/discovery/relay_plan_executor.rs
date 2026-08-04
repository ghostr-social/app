//! nostr_sdk-backed plan executor: resolves each query's relay set
//! (search relays, outbox relays, or the bootstrap pool) and fans the
//! plan's queries out concurrently. Parity source:
//! lib/platform/nostr/ndk_nostr_video_event_query.dart.

use crate::discovery::outbox_directory::{max_outbox_relays, OutboxDirectory};
use crate::discovery::plan_executor::{PlanExecutor, PlanFailure, PlanFuture, PlannedRetrieval};
use crate::discovery::search_queries::{resolve_relays, OutboxLookup, PlannedQuery, QueryPlan, QueryRole};
use crate::engine::DataUsageLevel;
use log::warn;
use nostr_sdk::{Client, Event, PublicKey};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Shared, live-updating outbox directory; ingestion happens on the
/// subscription side, lookups here.
pub type SharedOutboxDirectory = Arc<RwLock<OutboxDirectory>>;

type FetchHandle = JoinHandle<Result<Vec<Event>, PlanFailure>>;

#[derive(Clone)]
pub struct RelayPlanExecutor {
    client: Arc<Client>,
    search_relays: Arc<[String]>,
    outbox: SharedOutboxDirectory,
    outbox_cap: usize,
}

impl RelayPlanExecutor {
    pub fn new(
        client: Arc<Client>,
        search_relays: Vec<String>,
        outbox: SharedOutboxDirectory,
        level: DataUsageLevel,
    ) -> Self {
        Self {
            client,
            search_relays: search_relays.into(),
            outbox,
            outbox_cap: max_outbox_relays(level),
        }
    }

    async fn run(self, plan: QueryPlan) -> Result<Vec<Event>, PlanFailure> {
        let outbox = self.outbox_relays(&plan.outbox).await;
        let fetches: Vec<(QueryRole, FetchHandle)> = plan
            .queries
            .into_iter()
            .map(|query| (query.role.clone(), self.spawn_fetch(query, outbox.as_deref())))
            .collect();
        collect_events(fetches).await
    }

    /// Dart `_outboxRelays`: an empty resolution falls back to the
    /// client's bootstrap pool.
    async fn outbox_relays(&self, lookup: &OutboxLookup) -> Option<Vec<String>> {
        let authors: &[PublicKey] = match lookup {
            OutboxLookup::Skip => return None,
            OutboxLookup::DiscoveryRelays => &[],
            OutboxLookup::AuthorWriteRelays(authors) => authors,
        };
        let relays = self
            .outbox
            .read()
            .await
            .relays_for_authors(authors, self.outbox_cap);
        if relays.is_empty() {
            None
        } else {
            Some(relays)
        }
    }

    fn spawn_fetch(&self, query: PlannedQuery, outbox: Option<&[String]>) -> FetchHandle {
        let relays = resolve_relays(&query.target, &self.search_relays, outbox);
        tokio::spawn(fetch(self.client.clone(), relays, query))
    }
}

impl PlanExecutor for RelayPlanExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        Box::pin(self.clone().run(retrieval.plan))
    }
}

/// The primary query's failure sinks the load; additive hiccups only
/// narrow the pool (Dart `_additiveEvents`).
async fn collect_events(
    fetches: Vec<(QueryRole, FetchHandle)>,
) -> Result<Vec<Event>, PlanFailure> {
    let mut events = Vec::new();
    for (role, fetch) in fetches {
        match (role, joined(fetch).await) {
            (_, Ok(fetched)) => events.extend(fetched),
            (QueryRole::Primary, Err(failure)) => return Err(failure),
            (QueryRole::Additive, Err(failure)) => {
                warn!("Skipping a failed additive discovery query: {}", failure.message);
            }
        }
    }
    Ok(events)
}

async fn joined(fetch: FetchHandle) -> Result<Vec<Event>, PlanFailure> {
    fetch
        .await
        .unwrap_or_else(|error| Err(PlanFailure::new(error.to_string())))
}

/// `None` relays query the bootstrap pool, like Dart passing no
/// explicit relays to ndk; explicit relays are ensured in the pool
/// before fetching.
async fn fetch(
    client: Arc<Client>,
    relays: Option<Vec<String>>,
    query: PlannedQuery,
) -> Result<Vec<Event>, PlanFailure> {
    let filters = vec![query.filter];
    let events = match relays {
        None => client.fetch_events(filters, query.timeout).await,
        Some(urls) => {
            ensure_relays(&client, &urls).await;
            client.fetch_events_from(urls, filters, query.timeout).await
        }
    }
    .map_err(|error| PlanFailure::new(error.to_string()))?;
    Ok(events.into_iter().collect())
}

async fn ensure_relays(client: &Client, urls: &[String]) {
    for url in urls {
        if client.add_relay(url).await.unwrap_or(false) {
            let _ = client.connect_relay(url).await;
        }
    }
}
