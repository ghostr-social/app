//! The FFI row carries protection separately from optional signed JSON.

use crate::api::feed::mapping::feed_post;
use crate::api::tests::support::{creator_profile, parsed_video_post};

#[test]
fn protected_original_carries_an_explicit_protection_bit() {
    let mut post = parsed_video_post(1, None);
    post.is_protected = true;

    let row = feed_post(&post, creator_profile(), None);

    assert!(row.is_protected);
    assert_eq!(row.signed_event_json, None);
}
