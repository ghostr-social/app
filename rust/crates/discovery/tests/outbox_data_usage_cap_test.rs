//! The outbox fan-out is capped before bootstrap merging, and the cap
//! per data-usage level mirrors `maxOutboxRelays` in
//! lib/features/settings/domain/data_usage_level.dart (6 / 12 / 18).

mod discovery_support;

use discovery_support::write_relay_list;
use ghostr_discovery::outbox::directory::{max_outbox_relays, OutboxDirectory};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Keys;

#[test]
fn mirrors_dart_max_outbox_relays_per_level() {
    assert_eq!(max_outbox_relays(DataUsageLevel::Conservative), 6);
    assert_eq!(max_outbox_relays(DataUsageLevel::Balanced), 12);
    assert_eq!(max_outbox_relays(DataUsageLevel::Aggressive), 18);
}

#[test]
fn caps_ranked_outbox_relays_at_each_level() {
    let alice = Keys::generate();
    let urls: Vec<String> = (0..20)
        .map(|index| format!("wss://relay-{index:02}.example"))
        .collect();
    let borrowed: Vec<&str> = urls.iter().map(String::as_str).collect();
    let mut directory = OutboxDirectory::new(Vec::new());
    directory.ingest(&write_relay_list(&alice, &borrowed, 10));

    for level in [
        DataUsageLevel::Conservative,
        DataUsageLevel::Balanced,
        DataUsageLevel::Aggressive,
    ] {
        let cap = max_outbox_relays(level);
        let relays = directory.relays_for_authors(&[alice.public_key()], cap);
        assert_eq!(relays.len(), cap, "cap for {level:?}");
        assert_eq!(relays, urls[..cap].to_vec(), "ranked slice for {level:?}");
    }
}

#[test]
fn bootstrap_relays_ride_above_the_cap() {
    // Dart caps the ranked outbox list before merging bootstrap, so the
    // final list may exceed the cap by the bootstrap relays.
    let alice = Keys::generate();
    let mut directory = OutboxDirectory::new(vec!["wss://boot.example".to_owned()]);
    directory.ingest(&write_relay_list(
        &alice,
        &["wss://a.example", "wss://b.example"],
        10,
    ));

    let relays = directory.relays_for_authors(&[alice.public_key()], 2);

    assert_eq!(
        relays,
        vec![
            "wss://boot.example".to_owned(),
            "wss://a.example".to_owned(),
            "wss://b.example".to_owned(),
        ],
    );
}

/// The main feed's lookup is capped the same way: the follows' ranked
/// write relays, cut to the level's `maxOutboxRelays`, after bootstrap.
#[test]
fn caps_the_follows_discovery_relays_at_each_level() {
    let follows: Vec<Keys> = (0..24).map(|_| Keys::generate()).collect();
    let mut directory = OutboxDirectory::new(vec!["wss://boot.example".to_owned()]);
    for (index, keys) in follows.iter().enumerate() {
        let url = format!("wss://write-{index:02}.example");
        directory.ingest(&write_relay_list(keys, &[url.as_str()], 10));
    }
    directory.track_viewer_follows(follows.iter().map(Keys::public_key).collect());

    for level in [
        DataUsageLevel::Conservative,
        DataUsageLevel::Balanced,
        DataUsageLevel::Aggressive,
    ] {
        let relays = directory.discovery_relays(max_outbox_relays(level));

        assert_eq!(
            relays.len(),
            max_outbox_relays(level) + 1,
            "cap for {level:?}"
        );
    }
}
