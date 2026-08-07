//! Broadcast relay selection: the author's declared write relays after
//! the bootstrap set, capped by the data-usage level
//! (`OutboxDirectory::relays_for_authors` semantics).

use crate::api::broadcast_control::broadcast_relays;
use crate::api::tests::feed_fixtures::relay_list_event;
use crate::discovery::outbox::directory::OutboxDirectory;
use crate::engine::DataUsageLevel;
use nostr_sdk::Keys;

#[test]
fn write_relays_follow_the_bootstrap_set() {
    let keys = Keys::generate();
    let mut directory = OutboxDirectory::new(vec!["wss://boot.example".to_owned()]);
    directory.ingest(&relay_list_event(&keys, &["wss://write.example"], 10));

    let relays = broadcast_relays(&directory, &keys.public_key(), DataUsageLevel::Balanced);
    assert_eq!(
        relays,
        vec![
            "wss://boot.example".to_owned(),
            "wss://write.example".to_owned()
        ]
    );
}

#[test]
fn an_unknown_author_broadcasts_to_the_bootstrap_set() {
    let directory = OutboxDirectory::new(vec!["wss://boot.example".to_owned()]);
    let relays = broadcast_relays(
        &directory,
        &Keys::generate().public_key(),
        DataUsageLevel::Balanced,
    );
    assert_eq!(relays, vec!["wss://boot.example".to_owned()]);
}
