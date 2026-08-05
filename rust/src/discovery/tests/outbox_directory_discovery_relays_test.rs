//! Relay coverage of the outbox lookup the main feed rides on.
//! Discovery ranks the follows' write relays after the bootstrap set.
//! The bootstrap task supplies the viewer's follow set, so a lookup
//! issued before kind-3 landed still benefits once it does.

use crate::discovery::outbox_directory::OutboxDirectory;
use nostr_sdk::prelude::*;

fn bootstrap() -> Vec<String> {
    vec![
        "wss://relay.damus.io".to_owned(),
        "wss://relay.snort.social".to_owned(),
        "wss://relay.nostr.band".to_owned(),
    ]
}

fn relay_list_event(keys: &Keys, url: &str) -> Event {
    EventBuilder::new(Kind::RelayList, "")
        .tags([Tag::parse(vec!["r".to_owned(), url.to_owned()]).expect("fixture tag")])
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(keys)
        .expect("fixture event")
}

#[test]
fn the_tracked_follows_write_relays_extend_the_bootstrap_set() {
    let follow = Keys::generate();
    let mut directory = OutboxDirectory::new(bootstrap());
    directory.ingest(&relay_list_event(&follow, "wss://follow.write"));
    directory.track_viewer_follows(vec![follow.public_key()]);

    let discovery = directory.discovery_relays(12);

    assert_eq!(discovery.len(), 4);
    assert_eq!(discovery.last(), Some(&"wss://follow.write".to_owned()));
}

/// Signed out — or before the viewer's kind-3 landed — there are no
/// follows to route by, so the feed degrades to the bootstrap set.
#[test]
fn a_viewer_without_tracked_follows_reaches_only_the_bootstrap_relays() {
    let follow = Keys::generate();
    let mut directory = OutboxDirectory::new(bootstrap());
    directory.ingest(&relay_list_event(&follow, "wss://follow.write"));

    assert_eq!(directory.discovery_relays(12), bootstrap());
}

/// A follow whose kind-10002 never arrived contributes nothing, exactly
/// like an author with no known relay list.
#[test]
fn a_follow_without_an_ingested_relay_list_adds_no_relays() {
    let mut directory = OutboxDirectory::new(bootstrap());
    directory.track_viewer_follows(vec![Keys::generate().public_key()]);

    assert_eq!(directory.discovery_relays(12), bootstrap());
}

#[test]
fn a_retrievals_relay_lists_are_ingested_as_one_batch() {
    let first = Keys::generate();
    let second = Keys::generate();
    let mut directory = OutboxDirectory::new(bootstrap());
    let events = [
        relay_list_event(&first, "wss://first.write"),
        relay_list_event(&second, "wss://second.write"),
    ];

    directory.ingest_all(&events);

    assert_eq!(
        directory.write_relays(&first.public_key()),
        ["wss://first.write"],
    );
    assert_eq!(
        directory.write_relays(&second.public_key()),
        ["wss://second.write"],
    );
}
