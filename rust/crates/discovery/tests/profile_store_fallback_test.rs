//! A creator with no kind-0 metadata still gets a full identity: the
//! first twelve npub characters plus an ellipsis as display name, the
//! full `@npub` handle, and no avatar.

mod feed_support;

use nostr_sdk::{Keys, ToBech32};
use ghostr_discovery::profile_store::ProfileStore;

#[test]
fn profile_store_falls_back_to_a_shortened_npub_identity() {
    let creator = Keys::generate().public_key();
    let npub = creator.to_bech32().expect("npub encodes");
    let store = ProfileStore::new();

    let profile = store.profile(&creator);

    assert_eq!(profile.display_name, format!("{}…", &npub[..12]));
    assert_eq!(profile.handle, format!("@{npub}"));
    assert_eq!(profile.avatar_url, None);
}

#[test]
fn profile_store_keeps_the_npub_handle_even_with_metadata() {
    let creator = Keys::generate();
    let npub = creator.public_key().to_bech32().expect("npub encodes");
    let mut store = ProfileStore::new();
    store.ingest(&feed_support::profile_event(
        &creator,
        r#"{"name":"alice","picture":"https://cdn.example/alice.png"}"#,
        10,
    ));

    let profile = store.profile(&creator.public_key());

    assert_eq!(profile.display_name, "alice");
    assert_eq!(profile.handle, format!("@{npub}"));
    assert_eq!(
        profile.avatar_url,
        Some("https://cdn.example/alice.png".to_owned()),
    );
}
