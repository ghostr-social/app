//! Write relays shared by more of the queried authors rank first, and
//! rank ties break by URL ascending — mirrors `_rankedWriteRelays` in
//! lib/platform/nostr/ndk_nostr_outbox_directory.dart.

mod discovery_support;

use discovery_support::write_relay_list;
use nostr_sdk::Keys;
use rust_lib_ghostr::discovery::outbox_directory::OutboxDirectory;

#[test]
fn ranks_relays_by_author_count_then_url() {
    let (alice, bob, carol) = (Keys::generate(), Keys::generate(), Keys::generate());
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(
        &alice,
        &["wss://shared.example", "wss://alice.example"],
        10,
    ));
    directory.ingest(&write_relay_list(
        &bob,
        &["wss://shared.example", "wss://tie-b.example"],
        10,
    ));
    directory.ingest(&write_relay_list(
        &carol,
        &["wss://shared.example", "wss://tie-a.example"],
        10,
    ));

    let authors = [alice.public_key(), bob.public_key(), carol.public_key()];
    let relays = directory.relays_for_authors(&authors, 12);

    assert_eq!(
        relays,
        vec![
            "wss://shared.example".to_owned(),
            "wss://alice.example".to_owned(),
            "wss://tie-a.example".to_owned(),
            "wss://tie-b.example".to_owned(),
        ],
    );
}

#[test]
fn counts_each_author_once_even_when_repeated() {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(&alice, &["wss://solo.example"], 10));
    directory.ingest(&write_relay_list(&bob, &["wss://duo.example"], 10));

    // Dart takes Set<NostrPublicKeyHex>, so a repeated author must not
    // inflate its relays' rank above another author's.
    let authors = [alice.public_key(), alice.public_key(), bob.public_key()];
    let relays = directory.relays_for_authors(&authors, 1);

    assert_eq!(relays, vec!["wss://duo.example".to_owned()]);
}
