mod feed_support;

use feed_support::{repost, video_note};
use ghostr_discovery::content::reposts::feed_post_from_event;
use nostr_sdk::Keys;

#[test]
fn verified_kind_six_preserves_original_and_records_outer_activity() {
    let original_keys = Keys::generate();
    let reposter_keys = Keys::generate();
    let original = video_note(&original_keys, "original", 10);
    let wrapper = repost(&reposter_keys, &original, 30);

    let post = feed_post_from_event(&wrapper).expect("verified repost parses");

    assert_eq!(post.event_id, original.id.to_hex());
    assert_eq!(post.author_pubkey, original.pubkey.to_hex());
    assert_eq!(post.created_at, 10);
    assert_eq!(post.feed_sort_at, 30);
    let provenance = post.repost.expect("outer provenance");
    assert_eq!(provenance.event_id, wrapper.id.to_hex());
    assert_eq!(provenance.reposter_pubkey, wrapper.pubkey.to_hex());
}
