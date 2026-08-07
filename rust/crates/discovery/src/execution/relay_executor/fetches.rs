use super::RelayPlanExecutor;
use crate::execution::fetch::{fetch, RelayFetch};
use crate::execution::collector::FetchHandle;
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::EventProgress;
use crate::query::search::{resolve_relays, PlannedQuery, QueryPlan, QueryRole};
use crate::session_generation::SessionGeneration;
use std::sync::Arc;

pub(super) struct ContentFetchRequest {
    pub session: SessionGeneration,
    pub plan: QueryPlan,
    pub outboxes: Vec<Option<Vec<String>>>,
    pub route: Arc<RelayPoolRoute>,
    pub progress: Option<EventProgress>,
}

pub(super) struct FetchRequest {
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
                    route: request.route.clone(),
                    progress: request.progress.clone(),
                });
                (role, fetch)
            })
            .collect()
    }

    pub(super) fn spawn_fetch(&self, request: FetchRequest) -> FetchHandle {
        let relays = resolve_relays(
            &request.query.target,
            &self.search_relays(),
            request.outbox.as_deref(),
        );
        tokio::spawn(fetch(RelayFetch {
            route: request.route,
            cache: self.cache.clone(),
            session: request.session,
            relays,
            query: request.query,
            progress: request.progress,
        }))
    }
}
