//! Late relay lists and follows cannot refill a reset outbox directory.

use crate::outbox_directory::OutboxDirectory;
use crate::session_generation::SessionGeneration;
use crate::tests::outbox_support::{relay_list_event, BOOTSTRAP_RELAY};
use nostr_sdk::Keys;

#[test]
fn stale_session_updates_are_rejected_after_reset() {
    let stale = SessionGeneration::initial();
    let fresh = stale.next();
    let old_author = Keys::generate();
    let new_author = Keys::generate();
    let mut directory = OutboxDirectory::new(vec![BOOTSTRAP_RELAY.to_owned()]);
    directory.ingest(&relay_list_event(&old_author, "wss://old.example"));
    directory.track_viewer_follows(vec![old_author.public_key()]);

    directory.reset_session(fresh);
    directory.ingest_for(
        stale,
        &relay_list_event(&old_author, "wss://late-old.example"),
    );
    directory.track_viewer_follows_for(stale, vec![old_author.public_key()]);

    assert_eq!(
        directory.discovery_relays(12),
        vec![BOOTSTRAP_RELAY.to_owned()]
    );
    assert!(directory.write_relays(&old_author.public_key()).is_empty());
    directory.ingest_for(fresh, &relay_list_event(&new_author, "wss://new.example"));
    assert_eq!(
        directory.write_relays(&new_author.public_key()),
        ["wss://new.example"]
    );
}
