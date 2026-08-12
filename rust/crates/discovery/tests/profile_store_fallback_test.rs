//! A creator with no kind-0 metadata still gets a full identity: the
//! first twelve npub characters plus an ellipsis as display name, the
//! full `@npub` handle, and no avatar.

mod feed_support;

use ghostr_discovery::content::profiles::ProfileStore;
use nostr_sdk::{Keys, ToBech32};

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
fn profile_store_uses_the_metadata_name_as_handle() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();
    store.ingest(&feed_support::profile_event(
        &creator,
        r#"{"name":"Alice_42","picture":"https://cdn.example/alice.png"}"#,
        10,
    ));

    let profile = store.profile(&creator.public_key());

    assert_eq!(profile.display_name, "Alice_42");
    assert_eq!(profile.handle, "@Alice_42");
    assert_eq!(
        profile.avatar_url,
        Some("https://cdn.example/alice.png".to_owned()),
    );
}
