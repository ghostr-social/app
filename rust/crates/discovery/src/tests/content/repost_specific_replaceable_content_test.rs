use crate::content::repost_resolution::feed_posts_from_events;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag, video};
use nostr_sdk::{Keys, Kind};

#[test]
fn specific_replaceable_repost_requires_the_signed_event_content() {
    let creator = Keys::generate();
    let original = video(&creator, Kind::Custom(34235), vec![tag(&["d", "clip"])]);
    let wrapper = signed_wrapper(
        &Keys::generate(),
        16,
        "",
        vec![
            tag(&["e", &original.id.to_hex()]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "34235"]),
        ],
    );
    let original_id = original.id.to_hex();

    let posts = feed_posts_from_events(&[original, wrapper]);

    assert!(posts.iter().any(|post| post.event_id == original_id));
    assert!(posts.iter().all(|post| post.repost.is_none()));
}
