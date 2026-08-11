//! Only well-formed NIP-65 write declarations are ingested: malformed
//! r tags, non-relay URLs, and read-only markers are skipped — mirrors
//! ndk's Nip65.fromEvent plus the `RelayUrl.tryParse` validation in
//! `_writeUrls` of lib/platform/nostr/ndk_nostr_outbox_directory.dart.

mod discovery_support;

use discovery_support::{list_event, r_tag, r_tag_marked, relay_list};
use ghostr_discovery::outbox::directory::OutboxDirectory;
use nostr_sdk::{Keys, Kind};

fn ingested(keys: &Keys, tags: Vec<Vec<String>>) -> Vec<String> {
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&relay_list(keys, tags, 10));
    directory.relays_for_authors(&[keys.public_key()], 12)
}

#[test]
fn skips_malformed_and_non_write_declarations() {
    let alice = Keys::generate();

    let relays = ingested(
        &alice,
        vec![
            vec!["r".to_owned()],                                  // no url
            vec!["e".to_owned(), "wss://note.example".to_owned()], // not an r tag
            r_tag("https://web.example"),                          // wrong scheme
            r_tag("ws://insecure.example"),                        // ws off localhost
            r_tag("wss://query.example?limit=1"),                  // query rejected
            r_tag("wss://fragment.example#top"),                   // fragment rejected
            r_tag("wss://bad-port.example:relay"),                 // port not numeric
            r_tag("wss://"),                                       // empty host
            r_tag("wss://-hyphen.example"),                        // invalid host edge
            r_tag_marked("wss://read-only.example", "read"),       // not a write relay
            r_tag("wss://kept.example"),
        ],
    );

    assert_eq!(relays, vec!["wss://kept.example".to_owned()]);
}

#[test]
fn keeps_write_and_unknown_markers_and_local_ws() {
    let alice = Keys::generate();

    // ndk treats an unknown marker as read+write, so it stays a write
    // relay; ws:// is valid only for local development hosts.
    let relays = ingested(
        &alice,
        vec![
            r_tag_marked("wss://writer.example", "write"),
            r_tag_marked("wss://banana.example", "banana"),
            r_tag("ws://localhost:7777"),
        ],
    );

    assert_eq!(
        relays,
        vec![
            "ws://localhost:7777".to_owned(),
            "wss://banana.example".to_owned(),
            "wss://writer.example".to_owned(),
        ],
    );
}

#[test]
fn ignores_relay_declarations_on_other_kinds() {
    let alice = Keys::generate();
    let mut directory = OutboxDirectory::new(Vec::new());
    let wrong_kind = list_event(
        &alice,
        Kind::ContactList,
        vec![r_tag("wss://not-nip65.example")],
        10,
    );
    directory.ingest(&wrong_kind);

    assert!(directory
        .relays_for_authors(&[alice.public_key()], 12)
        .is_empty());
}
