//! Additive kind-0 lookup for the creators returned by a feed query.

use super::RelayPlanExecutor;
use crate::content::parsing::ParsedVideoPost;
use crate::content::repost_resolution::feed_posts_from_events;
use crate::execution::collector::collect_best_effort_events;
use crate::query::events::plan_event_queries;
use crate::query::search::QueryPlan;
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
        let fetches = self.enrichment_fetches(session, plan, outboxes, route);
        events.extend(collect_best_effort_events(fetches).await);
        Ok(events)
    }
}

fn profile_plan(events: &[Event]) -> Option<QueryPlan> {
    let posts = feed_posts_from_events(events);
    let authors: BTreeSet<_> = posts.iter().flat_map(profile_authors).collect();
    if authors.is_empty() {
        return None;
    }
    let filter = Filter::new().kind(Kind::Metadata).authors(authors);
    Some(plan_event_queries(vec![filter]))
}

fn profile_authors(post: &ParsedVideoPost) -> Vec<nostr_sdk::PublicKey> {
    let mut authors = Vec::new();
    if let Ok(author) = nostr_sdk::PublicKey::from_hex(&post.author_pubkey) {
        authors.push(author);
    }
    if let Some(reposter) = post
        .repost
        .as_ref()
        .and_then(|repost| nostr_sdk::PublicKey::from_hex(&repost.reposter_pubkey).ok())
    {
        authors.push(reposter);
    }
    authors
}
