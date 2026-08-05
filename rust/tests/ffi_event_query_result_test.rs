//! Generic read results expose the complete Dart domain event record.

use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::api::event_types::FfiNostrEvent;

#[test]
fn ffi_event_preserves_identity_tags_content_and_time() {
    let event = EventBuilder::new(Kind::Reaction, "+")
        .tag(Tag::parse(["A", "34236:author:clip", "root"]).expect("tag"))
        .custom_created_at(Timestamp::from(1_700_000_123))
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let mapped = FfiNostrEvent::from(&event);

    assert_eq!(mapped.id, event.id.to_hex());
    assert_eq!(mapped.pubkey, event.pubkey.to_hex());
    assert_eq!(mapped.kind, 7);
    assert_eq!(
        mapped.tags,
        vec![vec![
            "A".to_owned(),
            "34236:author:clip".to_owned(),
            "root".to_owned(),
        ]]
    );
    assert_eq!(mapped.content, "+");
    assert_eq!(mapped.created_at, 1_700_000_123);
}
