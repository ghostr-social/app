//! Bootstrap relays always lead the result and outbox relays merge in
//! after them without duplicates; authors with no known relay list fall
//! back to the bootstrap set alone — mirrors `_merged` and `_guarded` in
//! `lib/platform/nostr/ndk_nostr_outbox_directory.dart`.

use crate::outbox::directory::OutboxDirectory;
use crate::tests::discovery_support::write_relay_list;
use nostr_sdk::Keys;

fn bootstrap() -> Vec<String> {
    vec![
        "wss://boot-one.example".to_owned(),
        "wss://boot-two.example".to_owned(),
    ]
}

#[test]
fn unknown_authors_resolve_to_bootstrap_relays() {
    let directory = OutboxDirectory::new(bootstrap());

    let relays = directory.relays_for_authors(&[Keys::generate().public_key()], 12);

    assert_eq!(relays, bootstrap());
}

#[test]
fn bootstrap_leads_and_deduplicates_against_outbox() {
    let alice = Keys::generate();
    let mut directory = OutboxDirectory::new(bootstrap());
    directory.ingest(&write_relay_list(
        &alice,
        &["wss://boot-two.example", "wss://alice.example"],
        10,
    ));

    let relays = directory.relays_for_authors(&[alice.public_key()], 12);

    assert_eq!(
        relays,
        vec![
            "wss://boot-one.example".to_owned(),
            "wss://boot-two.example".to_owned(),
            "wss://alice.example".to_owned(),
        ],
    );
}

#[test]
fn no_authors_resolve_to_bootstrap_relays() {
    let directory = OutboxDirectory::new(bootstrap());

    assert_eq!(directory.relays_for_authors(&[], 12), bootstrap());
}
