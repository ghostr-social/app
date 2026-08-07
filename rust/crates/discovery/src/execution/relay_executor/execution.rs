//! Executes one routed plan while retaining its wire-filter cursor.

use super::fetches::ContentFetchRequest;
use super::RelayPlanExecutor;
use crate::plan_executor::{PlanPage, PlannedRetrieval};
use crate::execution::collector::collect_page;
use crate::retrieval_types::{EventProgress, PlanFailure};
use crate::session_generation::SESSION_RESET_MESSAGE;
use nostr_sdk::Event;

impl RelayPlanExecutor {
    pub(super) async fn run(
        self,
        retrieval: PlannedRetrieval,
        progress: Option<EventProgress>,
    ) -> Result<Vec<Event>, PlanFailure> {
        self.run_page(retrieval, progress)
            .await
            .map(|page| page.events)
    }

    pub(super) async fn run_page(
        self,
        retrieval: PlannedRetrieval,
        progress: Option<EventProgress>,
    ) -> Result<PlanPage, PlanFailure> {
        let session = retrieval.context.session();
        let priority = retrieval.priority;
        let plan = retrieval.plan;
        let route = self.relay_pool.begin_route(session).await?;
        self.adopt_session_viewer(session, &plan).await?;
        let outboxes = self.session_plan_outboxes(session, &plan).await?;
        let fetches = self.content_fetches(ContentFetchRequest {
            session,
            plan,
            outboxes,
            route: route.clone(),
            progress,
        });
        let page = collect_page(fetches).await?;
        let events = self
            .enrich_profiles(session, priority, page.events, route.clone())
            .await?;
        route.ensure_current()?;
        if !self.cache.is_current(session).await {
            return Err(PlanFailure::new(SESSION_RESET_MESSAGE));
        }
        Ok(PlanPage {
            events,
            cursor: page.cursor,
        })
    }
}
