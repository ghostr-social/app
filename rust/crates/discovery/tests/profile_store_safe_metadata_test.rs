use crate::content::profiles::ProfileStore;
use crate::tests::feed_support::profile_event;
use nostr_sdk::Keys;

#[test]
fn relay_metadata_is_bounded_and_safe_to_render() {
    let creator = Keys::generate();
    let mut store = ProfileStore::new();
    store.ingest(&profile_event(
        &creator,
        r#"{"display_name":"  Nora\u202eRelay\nAdmin  ","name":"@Nora.Dev\u0000","picture":"https://user:secret@example.com/p.png"}"#,
        10,
    ));

    let profile = store.profile(&creator.public_key());

    assert_eq!(profile.display_name, "Nora Relay Admin");
    assert_eq!(profile.handle, "@Nora.Dev");
    assert_eq!(profile.avatar_url, None);
}
