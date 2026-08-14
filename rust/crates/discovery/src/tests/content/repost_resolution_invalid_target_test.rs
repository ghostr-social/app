use crate::content::repost_resolution::feed_posts_from_events;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag, video};
use nostr_sdk::{Keys, Kind};

#[test]
fn invalid_signed_target_cannot_resolve_an_empty_repost() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let control = video(&Keys::generate(), Kind::Custom(21), vec![]);
    let control_id = control.id.to_hex();
    let mut original = video(&creator, Kind::Custom(21), vec![]);
    let wrapper = signed_wrapper(&reposter, 16, "", vec![tag(&["e", &original.id.to_hex()])]);
    original.content.push_str("?tampered");
    let posts = feed_posts_from_events(&[control, original, wrapper]);

    assert!(posts.iter().any(|post| post.event_id == control_id));
    assert!(posts.iter().all(|post| post.repost.is_none()));
}
