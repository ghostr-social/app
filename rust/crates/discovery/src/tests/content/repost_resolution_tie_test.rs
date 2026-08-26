use crate::content::repost_resolution::feed_posts_from_events;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag};
use nostr_sdk::{EventBuilder, Keys, Kind, Timestamp};

#[test]
fn coordinate_resolution_uses_the_smaller_id_on_a_timestamp_tie() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let first = addressable_video(&creator, "https://cdn.example/one.mp4");
    let second = addressable_video(&creator, "https://cdn.example/two.mp4");
    let winner = core::cmp::min(first.id, second.id).to_hex();
    let coordinate = format!("34235:{}:clip", creator.public_key());
    let wrapper = signed_wrapper(&reposter, 16, "", vec![tag(&["a", &coordinate])]);

    let posts = feed_posts_from_events(&[first, second, wrapper]);
    let repost = posts
        .iter()
        .find(|post| post.repost.is_some())
        .expect("repost");

    assert_eq!(repost.event_id, winner);
}

fn addressable_video(keys: &Keys, url: &str) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(34235), url)
        .tags([tag(&["d", "clip"])])
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(keys)
        .expect("video")
}
