use crate::content::repost_reference::reference_for_repost;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag, video};
use nostr_sdk::{JsonUtil as _, Keys, Kind};

#[test]
fn protected_embedded_video_has_no_repost_reference() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = video(&creator, Kind::Custom(21), vec![tag(&["-"])]);
    let wrapper = signed_wrapper(
        &reposter,
        16,
        original.as_json(),
        vec![tag(&["e", &original.id.to_hex()])],
    );

    assert!(reference_for_repost(&wrapper).is_none());
}
