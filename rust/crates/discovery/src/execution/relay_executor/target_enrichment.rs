//! Exact NIP-18 target lookup before deletion and profile enrichment.

use super::target_dependencies::{dependent_target_plan, DependentTargetPlan};
use super::RelayPlanExecutor;
use crate::execution::collector::{collect_partial_fetches, CollectedEvents};
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::{PlanFailure, RetrievalPriority};
use crate::session_generation::SessionGeneration;
use nostr_sdk::{Event, EventId};
use std::collections::BTreeSet;
use std::sync::Arc;

#[cfg(test)]
pub(crate) use super::target_dependencies::target_plan_with_dependencies;
#[cfg(test)]
pub(crate) use super::target_planning::target_plan;
#[cfg(test)]
pub(crate) use super::target_planning::MAX_TARGET_LOOKUPS;

pub(super) struct TargetEnrichment {
    pub(super) events: Vec<Event>,
    pub(super) retry: BTreeSet<EventId>,
}

impl RelayPlanExecutor {
    pub(super) async fn enrich_targets(
        &self,
        session: SessionGeneration,
        priority: RetrievalPriority,
        events: Vec<Event>,
        route: Arc<RelayPoolRoute>,
    ) -> Result<TargetEnrichment, PlanFailure> {
        if priority == RetrievalPriority::Enrichment {
            return Ok(complete(events));
        }
        let Some(dependent) = dependent_target_plan(&events) else {
            return Ok(complete(events));
        };
        let DependentTargetPlan {
            plan,
            dependencies,
            unplanned,
        } = dependent;
        let outboxes = self.session_plan_outboxes(session, &plan).await?;
        let fetches = self.enrichment_fetches(session, plan, outboxes, route);
        let outcomes = collect_partial_fetches(fetches).await;
        Ok(target_result(events, dependencies, unplanned, outcomes))
    }
}

fn complete(events: Vec<Event>) -> TargetEnrichment {
    TargetEnrichment {
        events,
        retry: BTreeSet::new(),
    }
}

fn target_result(
    mut events: Vec<Event>,
    dependencies: Vec<BTreeSet<EventId>>,
    unplanned: BTreeSet<EventId>,
    outcomes: Vec<CollectedEvents>,
) -> TargetEnrichment {
    let mut retry = unplanned;
    if outcomes.len() != dependencies.len() {
        retry.extend(dependencies.into_iter().flatten());
        return TargetEnrichment { events, retry };
    }
    for (outcome, dependencies) in outcomes.into_iter().zip(dependencies) {
        if !outcome.complete {
            retry.extend(dependencies);
        }
        events.extend(outcome.events);
    }
    TargetEnrichment { events, retry }
}
