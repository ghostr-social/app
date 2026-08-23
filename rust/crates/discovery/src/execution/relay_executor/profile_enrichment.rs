//! Additive kind-0 lookup for the creators returned by a feed query.

use super::RelayPlanExecutor;
use crate::content::blossom::supports_blossom;
use crate::content::parsing::ParsedVideoPost;
use crate::content::repost_resolution::feed_posts_from_events;
use crate::execution::collector::collect_best_effort_events;
use crate::feed::store::{compact_occurrences, QUERY_POST_RETENTION};
use crate::query::events::plan_event_queries;
use crate::query::search::QueryPlan;
use crate::relay::route::RelayPoolRoute;
use crate::retrieval_types::{PlanFailure, RetrievalPriority};
use crate::session_generation::SessionGeneration;
use nostr_sdk::{Event, Filter, Kind};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(crate) const MAX_PROFILE_OCCURRENCES: usize = QUERY_POST_RETENTION;
pub(crate) const MAX_PROFILE_AUTHORS: usize = MAX_PROFILE_OCCURRENCES * 2;
pub(crate) const MAX_PROFILE_AUTHORS_PER_QUERY: usize = 1_000;

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

pub(crate) fn profile_plan(events: &[Event]) -> Option<QueryPlan> {
    let posts = profile_occurrences(events);
    let authors: BTreeSet<_> = posts.iter().flat_map(profile_authors).collect();
    if authors.is_empty() {
        return None;
    }
    let media_authors: BTreeSet<_> = posts
        .iter()
        .filter(|post| supports_blossom(post))
        .filter_map(|post| nostr_sdk::PublicKey::from_hex(&post.author_pubkey).ok())
        .collect();
    debug_assert!(authors.len() <= MAX_PROFILE_AUTHORS);
    let mut filters = author_filters(Kind::Metadata, authors);
    filters.extend(author_filters(Kind::Custom(10063), media_authors));
    Some(plan_event_queries(filters))
}

fn profile_occurrences(events: &[Event]) -> Vec<ParsedVideoPost> {
    let mut posts = feed_posts_from_events(events);
    compact_occurrences(&mut posts, MAX_PROFILE_OCCURRENCES);
    posts
}

fn author_filters(kind: Kind, authors: BTreeSet<nostr_sdk::PublicKey>) -> Vec<Filter> {
    let authors: Vec<_> = authors.into_iter().collect();
    authors
        .chunks(MAX_PROFILE_AUTHORS_PER_QUERY)
        .map(|chunk| {
            Filter::new()
                .kind(kind)
                .authors(chunk.iter().copied())
                .limit(chunk.len())
        })
        .collect()
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
