use super::RelayPlanExecutor;
use crate::execution::collector::FetchHandle;
use crate::execution::fetch::{fetch, RelayFetch};
use crate::query::search::{resolve_relays, PlannedQuery, QueryPlan, QueryRole};
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::EventProgress;
use crate::session_generation::SessionGeneration;
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout_at, Instant};

const ENRICHMENT_FETCH_CONCURRENCY: usize = 8;
const ENRICHMENT_STAGE_TIMEOUT: Duration = Duration::from_secs(6);
const ENRICHMENT_TIMEOUT_MESSAGE: &str = "relay enrichment timed out";

pub(super) struct ContentFetchRequest {
    pub session: SessionGeneration,
    pub plan: QueryPlan,
    pub outboxes: Vec<Option<Vec<String>>>,
    pub route: Arc<RelayPoolRoute>,
    pub progress: Option<EventProgress>,
}

struct FetchRequest {
    pub session: SessionGeneration,
    pub query: PlannedQuery,
    pub outbox: Option<Vec<String>>,
    pub route: Arc<RelayPoolRoute>,
    pub progress: Option<EventProgress>,
}

impl RelayPlanExecutor {
    pub(super) fn content_fetches(
        &self,
        request: ContentFetchRequest,
    ) -> Vec<(QueryRole, FetchHandle)> {
        request
            .plan
            .queries
            .into_iter()
            .zip(request.outboxes)
            .map(|(query, outbox)| {
                let role = query.role.clone();
                let fetch = self.spawn_fetch(FetchRequest {
                    session: request.session,
                    query,
                    outbox,
                    route: std::sync::Arc::clone(&request.route),
                    progress: request.progress.clone(),
                });
                (role, fetch)
            })
            .collect()
    }

    fn spawn_fetch(&self, request: FetchRequest) -> FetchHandle {
        tokio::spawn(fetch(self.relay_fetch(request)))
    }

    fn relay_fetch(&self, request: FetchRequest) -> RelayFetch {
        let relays = resolve_relays(
            &request.query.target,
            &self.search_relays(),
            request.outbox.as_deref(),
        );
        RelayFetch {
            route: request.route,
            cache: std::sync::Arc::clone(&self.cache),
            session: request.session,
            relays,
            query: request.query,
            progress: request.progress,
        }
    }

    pub(super) fn enrichment_fetches(
        &self,
        session: SessionGeneration,
        plan: QueryPlan,
        outboxes: Vec<Option<Vec<String>>>,
        route: &Arc<RelayPoolRoute>,
    ) -> Vec<(QueryRole, FetchHandle)> {
        let gate = Arc::new(Semaphore::new(ENRICHMENT_FETCH_CONCURRENCY));
        let deadline = Instant::now() + ENRICHMENT_STAGE_TIMEOUT;
        plan.queries
            .into_iter()
            .zip(outboxes)
            .map(|(query, outbox)| {
                let request = FetchRequest {
                    session,
                    query,
                    outbox,
                    route: Arc::clone(route),
                    progress: None,
                };
                let fetch = self.spawn_gated_fetch(request, std::sync::Arc::clone(&gate), deadline);
                (QueryRole::Additive, fetch)
            })
            .collect()
    }

    fn spawn_gated_fetch(
        &self,
        request: FetchRequest,
        gate: Arc<Semaphore>,
        deadline: Instant,
    ) -> FetchHandle {
        let request = self.relay_fetch(request);
        tokio::spawn(async move {
            let permit = timeout_at(deadline, gate.acquire_owned())
                .await
                .map_err(|error| {
                    log::warn!("Enrichment permit timed out: {error}");
                    crate::retrieval_types::PlanFailure::new(ENRICHMENT_TIMEOUT_MESSAGE)
                })?
                .map_err(|error| crate::retrieval_types::PlanFailure::new(error.to_string()))?;
            let result = timeout_at(deadline, fetch(request))
                .await
                .map_err(|error| {
                    log::warn!("Enrichment fetch timed out: {error}");
                    crate::retrieval_types::PlanFailure::new(ENRICHMENT_TIMEOUT_MESSAGE)
                })?;
            drop(permit);
            result
        })
    }
}
