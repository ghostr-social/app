use crate::content::repost_reference::reference_for_repost;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag};
use nostr_sdk::Keys;

#[test]
fn ambiguous_duplicate_repost_references_are_rejected() {
    let keys = Keys::generate();
    let event = signed_wrapper(
        &keys,
        16,
        "",
        vec![tag(&["e", &"0".repeat(64)]), tag(&["e", &"1".repeat(64)])],
    );
    let address = signed_wrapper(
        &keys,
        16,
        "",
        vec![
            tag(&["a", &format!("34235:{}:one", keys.public_key())]),
            tag(&["a", &format!("34235:{}:two", keys.public_key())]),
        ],
    );

    assert!(reference_for_repost(&event).is_none());
    assert!(reference_for_repost(&address).is_none());
}
