//! The retrievals that populate the outbox directory and the social
//! graph: the viewer's own lists and the relay lists of the pubkeys a
//! feed cares about. Pure plans — [`crate::discovery::outbox_bootstrap`]
//! runs them. Rust stand-in for ndk's `getContactList` plus
//! `loadMissingRelayListsFromNip65OrNip02`
//! (lib/platform/nostr/ndk_nostr_outbox_directory.dart).

use crate::discovery::event_cache::ViewerScope;
use crate::discovery::search_queries::{
    OutboxLookup, PlannedQuery, QueryPlan, QueryRole, RelayTarget, FEED_QUERY_TIMEOUT,
};
use nostr_sdk::{Filter, Kind, PublicKey};

/// The viewer's own replaceable lists: kind-3 follows (routing),
/// kind-10000 mutes (filtering), kind-10002 relays (their own outbox).
pub const VIEWER_LIST_KINDS: [u16; 3] = [3, 10_000, 10_002];

/// NIP-65 relay list.
pub const RELAY_LIST_KIND: u16 = 10_002;

/// How many authors one relay-list query may name. Relay lists are
/// replaceable, so the wire limit equals the author count and a filter
/// naming every follow of a large account would be rejected by many
/// relays; the bootstrap chases the rest on later batches.
pub const MAX_RELAY_LIST_AUTHORS: usize = 100;

/// Everything the viewer publishes about themselves, in one query.
pub fn viewer_lists_plan(viewer: PublicKey) -> QueryPlan {
    let filter = Filter::new()
        .kinds(VIEWER_LIST_KINDS.iter().copied().map(Kind::from))
        .author(viewer)
        .limit(VIEWER_LIST_KINDS.len());
    plan(filter)
}

/// The relay lists of a named set of authors: the viewer's follows, or
/// the creators a profile feed just opened.
pub fn author_relay_lists_plan(authors: &[PublicKey]) -> QueryPlan {
    let filter = Filter::new()
        .kind(Kind::from(RELAY_LIST_KIND))
        .authors(authors.iter().copied())
        .limit(authors.len());
    plan(filter)
}

/// Relay-list work rides the same relays a feed would use — bootstrap
/// until the directory knows better, the follows' relays afterwards —
/// and gives up on the feed timeout so a quiet relay never pins a task.
/// It claims no viewer, so a relay-list chase never rescopes the
/// session's event pool.
fn plan(filter: Filter) -> QueryPlan {
    QueryPlan {
        outbox: OutboxLookup::DiscoveryRelays,
        viewer: ViewerScope::Unknown,
        queries: vec![PlannedQuery {
            filter,
            target: RelayTarget::OutboxRelays,
            role: QueryRole::Primary,
            timeout: FEED_QUERY_TIMEOUT,
        }],
    }
}
