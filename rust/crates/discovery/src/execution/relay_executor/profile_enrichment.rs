//! Additive kind-0 lookup for the creators returned by a feed query.

use super::fetches::FetchRequest;
use super::RelayPlanExecutor;
use crate::content::parsing::video_post_from_event;
use crate::execution::collector::{collect_events, FetchHandle};
use crate::query::events::plan_event_queries;
use crate::query::search::{QueryPlan, QueryRole};
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::{PlanFailure, RetrievalPriority};
use crate::session_generation::SessionGeneration;
use nostr_sdk::{Event, Filter, Kind};
use std::collections::BTreeSet;
use std::sync::Arc;

impl RelayPlanExecutor {
    pub(super) async fn enrich_profiles(
        &self,
        session: SessionGeneration,
        priority: RetrievalPriority,
        mut events: Vec<Event>,
        route: Arc<RelayPoolRoute>,
    ) -> Result<Vec<Event>, PlanFailure> {
        if priority == RetrievalPriority::Enrichment {
            return Ok(events);
        }
        let Some(plan) = profile_plan(&events) else {
            return Ok(events);
        };
        let outboxes = self.session_plan_outboxes(session, &plan).await?;
        let fetches = self.profile_fetches(session, plan, outboxes, route);
        events.extend(collect_events(fetches).await?);
        Ok(events)
    }

    fn profile_fetches(
        &self,
        session: SessionGeneration,
        plan: QueryPlan,
        outboxes: Vec<Option<Vec<String>>>,
        route: Arc<RelayPoolRoute>,
    ) -> Vec<(QueryRole, FetchHandle)> {
        plan.queries
            .into_iter()
            .zip(outboxes)
            .map(|(query, outbox)| {
                let fetch = self.spawn_fetch(FetchRequest {
                    session,
                    query,
                    outbox,
                    route: route.clone(),
                    progress: None,
                });
                (QueryRole::Additive, fetch)
            })
            .collect()
    }
}

fn profile_plan(events: &[Event]) -> Option<QueryPlan> {
    let authors: BTreeSet<_> = events
        .iter()
        .filter_map(|event| video_post_from_event(event).map(|_| event.pubkey))
        .collect();
    if authors.is_empty() {
        return None;
    }
    let filter = Filter::new().kind(Kind::Metadata).authors(authors);
    Some(plan_event_queries(vec![filter]))
}
