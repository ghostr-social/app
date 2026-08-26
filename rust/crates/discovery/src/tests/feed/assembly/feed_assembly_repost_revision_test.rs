use crate::content::reposts::feed_post_from_event;
use crate::feed::assembly::test_support::canonical_posts;
use crate::tests::feed_support::{addressable_video, parsed, repost};
use nostr_sdk::Keys;

#[test]
fn recent_repost_of_stale_revision_uses_latest_original_media() {
    let creator = Keys::generate();
    let stale = addressable_video(&creator, "clip", "stale", 10);
    let current = addressable_video(&creator, "clip", "current", 20);
    let wrapper = repost(&Keys::generate(), &stale, 30);
    let occurrences = vec![
        parsed(&current),
        feed_post_from_event(&wrapper).expect("repost parses"),
    ];

    let posts = canonical_posts(occurrences);

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].event_id, current.id.to_hex());
    assert_eq!(posts[0].meta.urls, ["https://cdn.example/current.mp4"]);
    assert_eq!(posts[0].feed_sort_at, 30);
    assert_eq!(
        posts[0]
            .repost
            .as_ref()
            .expect("valid test fixture")
            .event_id,
        wrapper.id.to_hex()
    );
}
