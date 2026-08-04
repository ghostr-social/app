//! Relay routing for discovery queries: which relay set answers which
//! filter, and in what role. Pure data the scheduler executes. Parity
//! sources: lib/platform/nostr/ndk_nostr_video_event_query.dart
//! (targeting, roles, timeouts) and
//! lib/features/settings/domain/app_settings.dart (the NIP-50 search
//! relay set, wired in via production_nostr_services.dart).

use std::time::Duration;

use nostr_sdk::{Filter, PublicKey};

use crate::discovery::video_filters::{discovery_filters, DiscoveryRequest};

/// Relays known to implement NIP-50 full-text search
/// (`AppSettings.defaultSearchRelays`).
pub const SEARCH_RELAY_URLS: [&str; 6] = [
    "wss://relay.nostr.band",
    "wss://nostr.wine",
    "wss://relay.noswhere.com",
    "wss://search.nos.today",
    "wss://antiprimal.net",
    "wss://relay.ditto.pub",
];

/// Canonical feed queries give up quickly; the feed must stay fluid.
pub const FEED_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Search relays keep answering after the fast ones went quiet; the extra
/// seconds are where the long tail of matches comes from.
pub const DISCOVERY_QUERY_TIMEOUT: Duration = Duration::from_secs(8);

/// How the outbox directory serves a whole request; resolved once and
/// shared by every query in the plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxLookup {
    /// A request built around a viewer term never needs the outbox.
    Skip,
    /// No wanted authors: the directory's general discovery relays.
    DiscoveryRelays,
    /// Write relays of the authors the request asks for.
    AuthorWriteRelays(Vec<PublicKey>),
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
}

/// Relays x filters for one discovery request: pure data, no IO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    pub outbox: OutboxLookup,
    pub queries: Vec<PlannedQuery>,
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
    }
}

/// Concrete relay list for one target, given the configured search relays
/// and the outbox lookup's result. `None` means the client's bootstrap
/// relay set answers (Dart passes no explicit relays to ndk).
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
    }
}

/// Discovery queries carry a NIP-50 term or tag filters
/// (`_isDiscovery` in ndk_nostr_video_event_query.dart).
fn is_discovery(filter: &Filter) -> bool {
    filter.search.is_some() || !filter.generic_tags.is_empty()
}

/// NIP-50 terms only work on relays that index for search. Tag-filtered
/// queries hit those deep indexes merged with the outbox; plain queries
/// route to the outbox relays where the wanted authors actually publish.
fn target_for(filter: &Filter) -> RelayTarget {
    if filter.search.is_some() {
        return RelayTarget::SearchRelays;
    }
    if filter.generic_tags.is_empty() {
        return RelayTarget::OutboxRelays;
    }
    RelayTarget::SearchAndOutboxRelays
}

/// Outbox relays serve the queries of a request that carries no term of
/// its own; a request built around a viewer term never needs them.
fn outbox_lookup(request: &DiscoveryRequest) -> OutboxLookup {
    if request.normalized_search().is_some() {
        return OutboxLookup::Skip;
    }
    if request.authors.is_empty() {
        return OutboxLookup::DiscoveryRelays;
    }
    OutboxLookup::AuthorWriteRelays(request.authors.clone())
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
