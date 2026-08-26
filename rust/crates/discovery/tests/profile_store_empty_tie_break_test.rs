use crate::content::profiles::ProfileStore;
use crate::tests::feed_support::profile_event;
use nostr_sdk::Keys;

#[test]
fn empty_equal_time_metadata_keeps_its_real_event_id() {
    let creator = Keys::generate();
    let empty = profile_event(&creator, "", 10);
    let named = (0..100)
        .map(|index| profile_event(&creator, &format!(r#"{{"name":"name{index}"}}"#), 10))
        .find(|candidate| candidate.id < empty.id)
        .expect("a lower deterministic signed event ID");
    let mut store = ProfileStore::new();

    store.ingest(&empty);
    store.ingest(&named);

    assert!(store
        .profile(&creator.public_key())
        .display_name
        .starts_with("name"));
}
