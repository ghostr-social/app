//! The relay-list retrievals the outbox bootstrap issues: the viewer's
//! own lists (kind-3 follows, kind-10000 mutes, kind-10002 relays) and
//! kind-10002 for a named set of authors — the Rust stand-in for ndk's
//! `getContactList` + `loadMissingRelayListsFromNip65OrNip02`.

use crate::discovery::outbox_plans::{author_relay_lists_plan, viewer_lists_plan};
use crate::discovery::search_queries::{OutboxLookup, QueryRole, RelayTarget};
use crate::discovery::tests::support::{author, filter_json, AUTHOR_A, AUTHOR_B};

#[test]
fn the_viewer_plan_asks_for_the_viewers_own_lists() {
    let plan = viewer_lists_plan(author(AUTHOR_A));

    let query = plan.queries.first().expect("the viewer plan runs one query");
    let json = filter_json(&query.filter);
    assert_eq!(json["kinds"], serde_json::json!([3, 10000, 10002]));
    assert_eq!(json["authors"], serde_json::json!([AUTHOR_A]));
    assert_eq!(query.role, QueryRole::Primary);
    assert_eq!(query.target, RelayTarget::OutboxRelays);
    assert_eq!(plan.outbox, OutboxLookup::DiscoveryRelays);
}

#[test]
fn the_author_plan_asks_only_for_relay_lists() {
    let plan = author_relay_lists_plan(&[author(AUTHOR_A), author(AUTHOR_B)]);

    let json = filter_json(&plan.queries[0].filter);
    assert_eq!(json["kinds"], serde_json::json!([10002]));
    assert_eq!(json["authors"], serde_json::json!([AUTHOR_A, AUTHOR_B]));
}

/// One replaceable list per author, so the limit is the author count —
/// a smaller limit would silently drop follows' relay lists.
#[test]
fn the_author_plan_leaves_room_for_every_authors_list() {
    let plan = author_relay_lists_plan(&[author(AUTHOR_A), author(AUTHOR_B)]);

    assert_eq!(filter_json(&plan.queries[0].filter)["limit"], 2);
}
