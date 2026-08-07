//! Kind-0 metadata is replaceable: strictly newer created_at wins, ties
//! and older revisions keep what is stored. Events whose content is not
//! a JSON object are ignored.

mod feed_support;

use feed_support::profile_event;
use nostr_sdk::Keys;
use ghostr_discovery::content::profiles::ProfileStore;

#[test]
fn profile_store_newest_metadata_wins_regardless_of_arrival_order() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();

    store.ingest(&profile_event(&creator, r#"{"name":"new"}"#, 20));
    store.ingest(&profile_event(&creator, r#"{"name":"old"}"#, 10));

    assert_eq!(store.profile(&creator.public_key()).display_name, "new");
}

#[test]
fn profile_store_ties_keep_the_stored_metadata() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();

    store.ingest(&profile_event(&creator, r#"{"name":"first"}"#, 10));
    store.ingest(&profile_event(&creator, r#"{"name":"second"}"#, 10));

    assert_eq!(store.profile(&creator.public_key()).display_name, "first");
}

#[test]
fn profile_store_drops_events_without_a_json_object_content() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(&creator, r#"{"name":"kept"}"#, 10));

    store.ingest(&profile_event(&creator, "not json at all", 20));
    store.ingest(&profile_event(&creator, r#"["a","list"]"#, 30));

    assert_eq!(store.profile(&creator.public_key()).display_name, "kept");
}

#[test]
fn profile_store_ignores_events_of_other_kinds() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();

    store.ingest(&feed_support::video_note(&creator, "clip", 10));

    let npub_short = &store.profile(&creator.public_key()).display_name;
    assert!(
        npub_short.starts_with("npub1"),
        "kept the fallback identity"
    );
}
