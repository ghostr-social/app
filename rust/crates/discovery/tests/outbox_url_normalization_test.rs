//! Relay URLs normalize before dedup and ranking: lowercase scheme and
//! host, default ports and trailing slashes stripped, and duplicate
//! urls within one event keep the last marker — mirrors ndk's
//! `cleanRelayUrl` composed with `RelayUrl.tryParse`
//! (lib/features/settings/domain/relay_url.dart).

mod discovery_support;

use discovery_support::{r_tag, r_tag_marked, relay_list};
use nostr_sdk::Keys;
use ghostr_discovery::outbox_directory::OutboxDirectory;

fn ingested(keys: &Keys, tags: Vec<Vec<String>>) -> Vec<String> {
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&relay_list(keys, tags, 10));
    directory.relays_for_authors(&[keys.public_key()], 12)
}

#[test]
fn normalizes_case_default_port_and_trailing_slash() {
    let alice = Keys::generate();

    let relays = ingested(
        &alice,
        vec![
            r_tag("WSS://Upper.Example/"),
            r_tag("wss://ported.example:443"),
            r_tag("wss://custom.example:8443"),
            r_tag("  wss://padded.example  "),
            // ndk rebuilds urls without credentials, so userinfo is
            // stripped rather than rejected.
            r_tag("wss://user@cred.example"),
        ],
    );

    assert_eq!(
        relays,
        vec![
            "wss://cred.example".to_owned(),
            "wss://custom.example:8443".to_owned(),
            "wss://padded.example".to_owned(),
            "wss://ported.example".to_owned(),
            "wss://upper.example".to_owned(),
        ],
    );
}

#[test]
fn duplicate_urls_in_one_event_keep_the_last_marker() {
    let alice = Keys::generate();

    // ndk builds a url -> marker map, so the later tag's marker wins.
    let demoted = ingested(
        &alice,
        vec![
            r_tag("wss://flip.example"),
            r_tag_marked("wss://flip.example/", "read"),
        ],
    );
    let promoted = ingested(
        &alice,
        vec![
            r_tag_marked("wss://flip.example", "read"),
            r_tag("wss://Flip.Example"),
        ],
    );

    assert!(demoted.is_empty());
    assert_eq!(promoted, vec!["wss://flip.example".to_owned()]);
}

#[test]
fn deduplicates_normalized_urls_within_one_event() {
    let alice = Keys::generate();

    let relays = ingested(
        &alice,
        vec![r_tag("wss://dup.example"), r_tag("WSS://dup.example/")],
    );

    assert_eq!(relays, vec!["wss://dup.example".to_owned()]);
}
