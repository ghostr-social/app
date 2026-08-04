//! nostr_sdk-backed plan executor: resolves each query's relay set
//! (search relays, outbox relays, or the bootstrap pool) and fans the
//! plan's queries out concurrently. Parity source:
//! lib/platform/nostr/ndk_nostr_video_event_query.dart.

use crate::discovery::event_cache::EventCache;
use crate::discovery::outbox_directory::{max_outbox_relays, OutboxDirectory};
use crate::discovery::plan_executor::{PlanExecutor, PlanFailure, PlanFuture, PlannedRetrieval};
use crate::discovery::search_queries::{resolve_relays, OutboxLookup, PlannedQuery, QueryPlan, QueryRole};
use crate::engine::DataUsageLevel;
use log::warn;
use nostr_sdk::{Client, Event};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_stream::{Stream, StreamExt};

/// Shared, live-updating outbox directory; ingestion happens on the
/// subscription side, lookups here.
pub type SharedOutboxDirectory = Arc<RwLock<OutboxDirectory>>;

type FetchHandle = JoinHandle<Result<Vec<Event>, PlanFailure>>;

#[derive(Clone)]
pub struct RelayPlanExecutor {
    client: Arc<Client>,
    /// The session event pool, shared with every clone: the client's own
    /// database, read back into each query's answer so a repeat query
    /// serves stored UNION network (ndk's cache parity).
    cache: Arc<EventCache>,
    search_relays: Arc<[String]>,
    outbox: SharedOutboxDirectory,
    /// Shared with every clone so a live data-usage change reaches the
    /// executor the scheduler already holds.
    outbox_cap: Arc<AtomicUsize>,
}

impl RelayPlanExecutor {
    pub fn new(
        client: Arc<Client>,
        search_relays: Vec<String>,
        outbox: SharedOutboxDirectory,
        level: DataUsageLevel,
    ) -> Self {
        Self {
            cache: Arc::new(EventCache::of(&client)),
            client,
            search_relays: search_relays.into(),
            outbox,
            outbox_cap: Arc::new(AtomicUsize::new(max_outbox_relays(level))),
        }
    }

    /// The session event pool this executor reads and files into.
    pub fn cache(&self) -> Arc<EventCache> {
        self.cache.clone()
    }

    /// Scopes the pool to whoever this plan's feed named, before any of
    /// its queries reads it; reports whether the pool was emptied.
    pub(crate) async fn adopt_plan_viewer(&self, plan: &QueryPlan) -> bool {
        self.cache.adopt(plan.viewer).await
    }

    /// Live outbox fan-out change (`ffi_set_delivery_config`): the next
    /// query of every open feed uses the new cap.
    pub fn set_data_usage(&self, level: DataUsageLevel) {
        self.outbox_cap
            .store(max_outbox_relays(level), Ordering::Relaxed);
    }

    async fn run(self, plan: QueryPlan) -> Result<Vec<Event>, PlanFailure> {
        self.adopt_plan_viewer(&plan).await;
        let outbox = self.outbox_relays(&plan.outbox).await;
        let fetches: Vec<(QueryRole, FetchHandle)> = plan
            .queries
            .into_iter()
            .map(|query| (query.role.clone(), self.spawn_fetch(query, outbox.as_deref())))
            .collect();
        collect_events(fetches).await
    }

    /// Dart `_outboxRelays`: the wanted authors' write relays, or — for
    /// a request that names none — the directory's discovery relays,
    /// which rank the viewer's follows' write relays
    /// (`discoveryRelayUrls`). An empty resolution falls back to the
    /// client's bootstrap pool.
    pub(crate) async fn outbox_relays(&self, lookup: &OutboxLookup) -> Option<Vec<String>> {
        let cap = self.outbox_cap.load(Ordering::Relaxed);
        let directory = self.outbox.read().await;
        let relays = match lookup {
            OutboxLookup::Skip => return None,
            OutboxLookup::DiscoveryRelays => directory.discovery_relays(cap),
            OutboxLookup::AuthorWriteRelays(authors) => directory.relays_for_authors(authors, cap),
        };
        if relays.is_empty() {
            None
        } else {
            Some(relays)
        }
    }

    fn spawn_fetch(&self, query: PlannedQuery, outbox: Option<&[String]>) -> FetchHandle {
        let relays = resolve_relays(&query.target, &self.search_relays, outbox);
        tokio::spawn(fetch(self.client.clone(), self.cache.clone(), relays, query))
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
/// before fetching. The relays' answer is filed in the session pool and
/// merged with what the pool already held for the same filter —
/// `stream_events*` reaches relays only (nostr-relay-pool 0.38
/// `pool/inner.rs`), so the union is assembled here.
async fn fetch(
    client: Arc<Client>,
    cache: Arc<EventCache>,
    relays: Option<Vec<String>>,
    query: PlannedQuery,
) -> Result<Vec<Event>, PlanFailure> {
    let filters = vec![query.filter.clone()];
    let streamed = match relays {
        None => client.stream_events(filters, query.timeout).await,
        Some(urls) => {
            ensure_relays(&client, &urls).await;
            client.stream_events_from(urls, filters, query.timeout).await
        }
    }
    .map_err(|error| PlanFailure::new(error.to_string()))?;
    let fetched = drain_events(streamed).await;
    Ok(cache.union(&query.filter, fetched).await)
}

/// Every event the relays streamed, in arrival order (the feed sorts
/// its own page). The sibling `fetch_events*` calls would collect into
/// `Events::new(&filters)`, a set capped at the single filter's
/// `limit`: that bounds the *union across relays* and drops the oldest,
/// while the wire filter already caps each relay on its own. ndk merges
/// the union unbounded, so draining is what keeps the pools the same
/// size. The pool deduplicates by event id before it streams.
pub(crate) async fn drain_events<S>(mut streamed: S) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    let mut events = Vec::new();
    while let Some(event) = streamed.next().await {
        events.push(event);
    }
    events
}

async fn ensure_relays(client: &Client, urls: &[String]) {
    for url in urls {
        if client.add_relay(url).await.unwrap_or(false) {
            let _ = client.connect_relay(url).await;
        }
    }
}
