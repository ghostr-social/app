//! Kind-10002 relay lists are replaceable: a strictly newer created_at
//! replaces the author's list, an older or equally old event is ignored
//! — the newest-wins floor mirrors `_newestContact`/`_newestMute` in
//! lib/platform/nostr/ndk_nostr_social_models.dart.

mod discovery_support;

use discovery_support::write_relay_list;
use nostr_sdk::Keys;
use ghostr_discovery::outbox::directory::OutboxDirectory;

fn relays_of(directory: &OutboxDirectory, keys: &Keys) -> Vec<String> {
    directory.relays_for_authors(&[keys.public_key()], 12)
}

#[test]
fn newer_list_replaces_the_previous_one() {
    let alice = Keys::generate();
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(&alice, &["wss://old.example"], 10));
    directory.ingest(&write_relay_list(&alice, &["wss://new.example"], 20));

    assert_eq!(
        relays_of(&directory, &alice),
        vec!["wss://new.example".to_owned()],
    );
}

#[test]
fn stale_list_arriving_late_is_ignored() {
    let alice = Keys::generate();
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(&alice, &["wss://new.example"], 20));
    directory.ingest(&write_relay_list(&alice, &["wss://old.example"], 10));

    assert_eq!(
        relays_of(&directory, &alice),
        vec!["wss://new.example".to_owned()],
    );
}

#[test]
fn equal_created_at_keeps_the_existing_list() {
    let alice = Keys::generate();
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(&alice, &["wss://first.example"], 10));
    directory.ingest(&write_relay_list(&alice, &["wss://second.example"], 10));

    assert_eq!(
        relays_of(&directory, &alice),
        vec!["wss://first.example".to_owned()],
    );
}

#[test]
fn lists_are_tracked_per_author() {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(&alice, &["wss://alice.example"], 20));
    directory.ingest(&write_relay_list(&bob, &["wss://bob.example"], 10));

    assert_eq!(
        relays_of(&directory, &bob),
        vec!["wss://bob.example".to_owned()],
    );
}
