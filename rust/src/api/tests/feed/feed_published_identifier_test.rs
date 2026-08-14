//! FFI keeps the exact addressable identifier used by Nostr coordinates.

use crate::api::feed::mapping::feed_post;
use crate::api::tests::support::{creator_profile, parsed_video_post};

#[test]
fn addressable_row_carries_the_exact_published_identifier() {
    let mut post = parsed_video_post(34_235, Some("clip"));
    post.published_identifier = Some(" clip ".to_owned());

    let row = feed_post(&post, creator_profile(), None);

    assert_eq!(row.identifier.as_deref(), Some("clip"));
    assert_eq!(row.published_identifier.as_deref(), Some(" clip "));
}
