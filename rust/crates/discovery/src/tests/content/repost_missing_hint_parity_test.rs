use crate::content::repost_reference::reference_for_repost;
use crate::content::reposts::feed_post_from_event;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag, video};
use nostr_sdk::{JsonUtil as _, Keys, Kind};

#[test]
fn kind_six_requires_a_hint_for_empty_and_embedded_forms() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = video(&creator, Kind::TextNote, vec![]);
    let reference = tag(&["e", &original.id.to_hex()]);
    let empty = signed_wrapper(&reposter, 6, "", vec![reference.clone()]);
    let embedded = signed_wrapper(&reposter, 6, original.as_json(), vec![reference]);

    assert!(reference_for_repost(&empty).is_none());
    assert!(feed_post_from_event(&embedded).is_none());
}
