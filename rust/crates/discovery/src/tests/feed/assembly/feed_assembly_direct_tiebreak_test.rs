use crate::content::reposts::feed_post_from_event;
use crate::feed::assembly::test_support::canonical_posts;
use crate::tests::feed_support::{parsed, repost, video_note};
use nostr_sdk::Keys;

#[test]
fn direct_occurrence_wins_a_same_time_tie_with_its_repost() {
    let creator = Keys::generate();
    let original = video_note(&creator, "clip", 10);
    let wrapper = repost(&Keys::generate(), &original, 10);
    let posts = canonical_posts(vec![
        parsed(&original),
        feed_post_from_event(&wrapper).expect("repost"),
    ]);

    assert_eq!(posts.len(), 1);
    assert!(posts[0].repost.is_none());
}
