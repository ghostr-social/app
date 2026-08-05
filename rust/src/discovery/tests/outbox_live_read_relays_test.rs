use crate::discovery::outbox_directory::OutboxDirectory;
use crate::discovery::tests::outbox_support::relay_list_event;
use nostr_sdk::Keys;

#[test]
fn configured_read_relays_are_replaced_without_losing_learned_routes() {
    let mut directory = OutboxDirectory::new(vec!["wss://old.example".to_owned()]);
    let author = Keys::generate();
    directory.ingest(&relay_list_event(&author, "wss://author.example"));

    directory.replace_bootstrap(vec!["wss://new.example".to_owned()]);

    assert_eq!(
        directory.relays_for_authors(&[author.public_key()], 12),
        vec!["wss://new.example", "wss://author.example"]
    );
}
