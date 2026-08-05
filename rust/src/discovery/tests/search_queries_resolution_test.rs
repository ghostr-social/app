//! Targets resolve to explicit relay lists: empty sets fall back to the
//! configured read relays (`None`), and merged targets dedupe with the
//! search relays first.

use crate::discovery::search_queries::{resolve_relays, RelayTarget};

fn relays(urls: &[&str]) -> Vec<String> {
    urls.iter().map(|url| url.to_string()).collect()
}

#[test]
fn search_target_uses_the_search_relays_or_bootstrap() {
    let search = relays(&["wss://a", "wss://b"]);

    let resolved = resolve_relays(&RelayTarget::SearchRelays, &search, None);

    assert_eq!(resolved, Some(search));
    assert_eq!(resolve_relays(&RelayTarget::SearchRelays, &[], None), None);
}

#[test]
fn outbox_target_uses_the_lookup_result_or_bootstrap() {
    let outbox = relays(&["wss://write"]);

    let resolved = resolve_relays(
        &RelayTarget::OutboxRelays,
        &relays(&["wss://a"]),
        Some(&outbox),
    );

    assert_eq!(resolved, Some(outbox));
    let unresolved = resolve_relays(&RelayTarget::OutboxRelays, &relays(&["wss://a"]), None);
    assert_eq!(unresolved, None);
}

#[test]
fn merged_target_dedupes_with_search_relays_first() {
    let search = relays(&["wss://a", "wss://b"]);
    let outbox = relays(&["wss://b", "wss://c"]);

    let resolved = resolve_relays(&RelayTarget::SearchAndOutboxRelays, &search, Some(&outbox));

    assert_eq!(resolved, Some(relays(&["wss://a", "wss://b", "wss://c"])));
}

#[test]
fn merged_target_with_nothing_to_merge_falls_back_to_bootstrap() {
    assert_eq!(
        resolve_relays(&RelayTarget::SearchAndOutboxRelays, &[], None),
        None
    );
}
