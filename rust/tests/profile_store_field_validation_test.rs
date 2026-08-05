//! Blank metadata clears optional fields; known fields must be strings.

mod feed_support;

use feed_support::profile_event;
use nostr_sdk::{Keys, ToBech32};
use rust_lib_ghostr::discovery::profile_store::ProfileStore;

#[test]
fn blank_metadata_replaces_named_profile_with_fallback() {
    let creator = Keys::generate();
    let expected = creator.public_key().to_bech32().expect("npub encodes");
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(
        &creator,
        r#"{"name":"old","picture":"https://old.example/avatar"}"#,
        10,
    ));

    store.ingest(&profile_event(&creator, "  ", 20));

    let profile = store.profile(&creator.public_key());
    assert_eq!(profile.display_name, format!("{}…", &expected[..12]));
    assert_eq!(profile.avatar_url, None);
}

#[test]
fn non_string_known_field_drops_the_metadata_event() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(&creator, r#"{"name":"kept"}"#, 10));

    store.ingest(&profile_event(&creator, r#"{"name":42}"#, 20));

    assert_eq!(store.profile(&creator.public_key()).display_name, "kept");
}
