//! Display-name precedence is `display_name` when non-blank, then `name`;
//! a name equal to the creator's hex public key is treated as missing.

mod feed_support;

use feed_support::profile_event;
use nostr_sdk::{Keys, ToBech32};
use ghostr_discovery::profile_store::ProfileStore;

fn display_name(content: &str) -> String {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(&creator, content, 10));
    store.profile(&creator.public_key()).display_name
}

#[test]
fn profile_store_prefers_display_name_over_name() {
    assert_eq!(
        display_name(r#"{"display_name":"Alice Prime","name":"alice"}"#),
        "Alice Prime",
    );
}

#[test]
fn profile_store_falls_back_to_name_when_display_name_is_blank() {
    assert_eq!(
        display_name(r#"{"display_name":"  ","name":"alice"}"#),
        "alice",
    );
}

#[test]
fn profile_store_short_npub_when_both_names_are_blank() {
    let creator = Keys::generate();
    let npub = creator.public_key().to_bech32().expect("npub encodes");
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(&creator, r#"{"name":"  "}"#, 10));

    let profile = store.profile(&creator.public_key());

    assert_eq!(profile.display_name, format!("{}…", &npub[..12]));
}

#[test]
fn profile_store_treats_a_hex_pubkey_name_as_no_name() {
    let creator = Keys::generate();
    let hex = creator.public_key().to_hex();
    let npub = creator.public_key().to_bech32().expect("npub encodes");
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(
        &creator,
        &format!(r#"{{"name":"{hex}"}}"#),
        10,
    ));

    let profile = store.profile(&creator.public_key());

    assert_eq!(profile.display_name, format!("{}…", &npub[..12]));
}
