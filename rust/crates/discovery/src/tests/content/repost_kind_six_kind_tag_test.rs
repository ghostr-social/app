use crate::content::repost_reference::reference_for_repost;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag};
use nostr_sdk::Keys;

#[test]
fn kind_six_rejects_an_incompatible_kind_tag() {
    let keys = Keys::generate();
    let wrapper = signed_wrapper(
        &keys,
        6,
        "",
        vec![
            tag(&["e", &"0".repeat(64), "wss://relay.example"]),
            tag(&["k", "21"]),
        ],
    );

    assert!(reference_for_repost(&wrapper).is_none());
}
