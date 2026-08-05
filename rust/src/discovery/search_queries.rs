//! Relay routing for discovery queries: which relay set answers each
//! filter, in what role, and under which timeout. Pure data executed by
//! the scheduler.

use std::time::Duration;

use nostr_sdk::{Filter, PublicKey};

use crate::discovery::event_cache::ViewerScope;
use crate::discovery::video_filters::{discovery_filters, DiscoveryRequest};

/// Canonical feed queries give up quickly; the feed must stay fluid.
pub const FEED_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Search relays keep answering after the fast ones went quiet; the extra
/// seconds are where the long tail of matches comes from.
pub const DISCOVERY_QUERY_TIMEOUT: Duration = Duration::from_secs(8);

/// A concrete outbox-directory lookup, used either as a plan default or
/// as one query's independent route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxLookup {
    /// A request built around a viewer term never needs the outbox.
    Skip,
    /// No wanted authors: the directory's general discovery relays.
    DiscoveryRelays,
    /// Write relays of the authors the request asks for.
    AuthorWriteRelays(Vec<PublicKey>),
}

/// Whether one query uses its request's shared outbox lookup or carries
/// a lookup that must be resolved independently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxRoute {
    Shared,
    Filter(OutboxLookup),
}

/// Which relay set executes one query. Resolution to concrete URLs
/// happens in [`resolve_relays`]; `None` there means bootstrap relays.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelayTarget {
    /// The NIP-50 search relay set alone.
    SearchRelays,
    /// Whatever the outbox lookup produced (bootstrap when it found none).
    OutboxRelays,
    /// Search relays' deep tag indexes merged with the outbox relays.
    SearchAndOutboxRelays,
}

/// Whether a query's failure sinks the load or only narrows the pool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryRole {
    /// The video-kind query: its failure fails the whole request.
    Primary,
    /// Note and file results only ever widen the pool; their hiccups must
    /// not sink the primary results.
    Additive,
}

/// One relay query the scheduler should execute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedQuery {
    pub filter: Filter,
    pub target: RelayTarget,
    pub role: QueryRole,
    pub timeout: Duration,
    pub outbox: OutboxRoute,
}

/// Relays x filters for one discovery request: pure data, no IO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    /// Default lookup for queries whose route is [`OutboxRoute::Shared`].
    pub outbox: OutboxLookup,
    pub queries: Vec<PlannedQuery>,
    /// Whose session the executor's event pool answers this plan from.
    pub viewer: ViewerScope,
}

/// Lays out the full query plan for one discovery request.
pub fn plan_discovery(request: &DiscoveryRequest) -> QueryPlan {
    let queries = discovery_filters(request)
        .into_iter()
        .enumerate()
        .map(|(index, filter)| planned(filter, index == 0))
        .collect();
    QueryPlan {
        outbox: outbox_lookup(request),
        queries,
        viewer: request.viewer,
    }
}

/// Concrete relay list for one target, given the configured search relays
/// and the outbox lookup's result. `None` means the owner's configured
/// read relays answer.
pub fn resolve_relays(
    target: &RelayTarget,
    search_relays: &[String],
    outbox_relays: Option<&[String]>,
) -> Option<Vec<String>> {
    match target {
        RelayTarget::SearchRelays => non_empty(search_relays.to_vec()),
        RelayTarget::OutboxRelays => outbox_relays.map(<[String]>::to_vec),
        RelayTarget::SearchAndOutboxRelays => {
            non_empty(merged(search_relays, outbox_relays.unwrap_or(&[])))
        }
    }
}

fn planned(filter: Filter, primary: bool) -> PlannedQuery {
    let timeout = if is_discovery(&filter) {
        DISCOVERY_QUERY_TIMEOUT
    } else {
        FEED_QUERY_TIMEOUT
    };
    PlannedQuery {
        target: target_for(&filter),
        role: if primary {
            QueryRole::Primary
        } else {
            QueryRole::Additive
        },
        timeout,
        filter,
        outbox: OutboxRoute::Shared,
    }
}

/// Discovery queries carry a NIP-50 term or generic tag filters.
pub(crate) fn is_discovery(filter: &Filter) -> bool {
    filter.search.is_some() || !filter.generic_tags.is_empty()
}

/// NIP-50 terms only work on relays that index for search. Tag-filtered
/// queries hit those deep indexes merged with the outbox; plain queries
/// route to the outbox relays where the wanted authors actually publish.
pub(crate) fn target_for(filter: &Filter) -> RelayTarget {
    if filter.search.is_some() {
        return RelayTarget::SearchRelays;
    }
    if filter.generic_tags.is_empty() {
        return RelayTarget::OutboxRelays;
    }
    RelayTarget::SearchAndOutboxRelays
}

/// Outbox relays serve the queries of a request that carries no term of
/// its own; a request built around a viewer term never needs them. The
/// routed authors are the ones the request filters by, or the
/// routing-only set the main feed carries — a request that knows
/// neither falls back to the directory's own discovery relays.
fn outbox_lookup(request: &DiscoveryRequest) -> OutboxLookup {
    if request.normalized_search().is_some() {
        return OutboxLookup::Skip;
    }
    let routed = request.routed_authors();
    if routed.is_empty() {
        return OutboxLookup::DiscoveryRelays;
    }
    OutboxLookup::AuthorWriteRelays(routed.to_vec())
}

fn merged(search: &[String], outbox: &[String]) -> Vec<String> {
    let mut relays = search.to_vec();
    for relay in outbox {
        if !relays.contains(relay) {
            relays.push(relay.clone());
        }
    }
    relays
}

fn non_empty(relays: Vec<String>) -> Option<Vec<String>> {
    if relays.is_empty() {
        None
    } else {
        Some(relays)
    }
}
