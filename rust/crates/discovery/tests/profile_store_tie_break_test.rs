mod feed_support;

use feed_support::profile_event;
use ghostr_discovery::content::profiles::ProfileStore;
use nostr_sdk::Keys;

#[test]
fn equal_time_metadata_uses_the_lowest_event_id() {
    let creator = Keys::generate();
    let first = profile_event(&creator, r#"{"name":"first"}"#, 10);
    let second = profile_event(&creator, r#"{"name":"second"}"#, 10);
    let (lower, higher, expected) = if first.id < second.id {
        (first, second, "first")
    } else {
        (second, first, "second")
    };
    let mut store = ProfileStore::new();

    store.ingest(&higher);
    store.ingest(&lower);

    assert_eq!(store.profile(&creator.public_key()).display_name, expected);
}
