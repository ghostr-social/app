use crate::content::parsing::MAX_REPOSTABLE_EVENT_BYTES;
use crate::content::repost_reference::reference_for_repost;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag};
use nostr_sdk::Keys;

#[test]
fn oversized_embedded_content_has_no_repost_reference() {
    let keys = Keys::generate();
    let wrapper = signed_wrapper(
        &keys,
        16,
        "x".repeat(MAX_REPOSTABLE_EVENT_BYTES + 1),
        vec![tag(&["e", &"0".repeat(64)])],
    );

    assert!(reference_for_repost(&wrapper).is_none());
}
