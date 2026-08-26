//! Targeted NIP-09 lookup with per-wrapper completion evidence.

use super::deletion_planning::{dependent_deletion_plan, DependentDeletionPlan};
use super::RelayPlanExecutor;
use crate::execution::collector::{collect_partial_fetches, CollectedEvents};
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::{PlanFailure, RetrievalPriority};
use crate::session_generation::SessionGeneration;
use nostr_sdk::{Event, EventId};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) struct DeletionEnrichment {
    pub(super) events: Vec<Event>,
    pub(super) settled: BTreeSet<EventId>,
}

impl RelayPlanExecutor {
    pub(super) async fn enrich_deletions(
        &self,
        session: SessionGeneration,
        priority: RetrievalPriority,
        events: Vec<Event>,
        route: Arc<RelayPoolRoute>,
    ) -> Result<DeletionEnrichment, PlanFailure> {
        if priority == RetrievalPriority::Enrichment {
            return Ok(without_settlement(events));
        }
        let Some(dependent) = dependent_deletion_plan(&events) else {
            return Ok(without_settlement(events));
        };
        let DependentDeletionPlan { plan, dependencies } = dependent;
        let outboxes = self.session_plan_outboxes(session, &plan).await?;
        let fetches = self.enrichment_fetches(session, plan, outboxes, &route);
        let outcomes = collect_partial_fetches(fetches).await;
        Ok(deletion_result(events, dependencies, outcomes))
    }
}

fn without_settlement(events: Vec<Event>) -> DeletionEnrichment {
    DeletionEnrichment {
        events,
        settled: BTreeSet::new(),
    }
}

fn deletion_result(
    mut events: Vec<Event>,
    dependencies: Vec<BTreeSet<EventId>>,
    outcomes: Vec<CollectedEvents>,
) -> DeletionEnrichment {
    let mut settled: BTreeSet<_> = dependencies.iter().flatten().copied().collect();
    if outcomes.len() != dependencies.len() {
        settled.clear();
        return DeletionEnrichment { events, settled };
    }
    for (outcome, dependencies) in outcomes.into_iter().zip(dependencies) {
        if !outcome.complete {
            settled.retain(|id| !dependencies.contains(id));
        }
        events.extend(outcome.events);
    }
    DeletionEnrichment { events, settled }
}

#[cfg(test)]
#[path = "deletion_enrichment_axiom_test.rs"]
pub(crate) mod axiom_test_support;
