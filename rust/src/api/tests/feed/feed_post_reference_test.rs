//! A feed row carries every field needed to rebuild its social reference:
//! event ID, creator public key, event kind, and optional identifier.

use crate::api::feed::mapping::feed_post;
use crate::api::tests::support::{creator_profile, parsed_video_post};

#[test]
fn addressable_rows_carry_the_kind_and_the_d_identifier() {
    let row = feed_post(
        &parsed_video_post(34_235, Some("clip")),
        creator_profile(),
        None,
    );

    assert_eq!(row.event_kind, 34_235);
    assert_eq!(row.identifier.as_deref(), Some("clip"));
}

#[test]
fn plain_rows_carry_their_kind_without_an_identifier() {
    let row = feed_post(&parsed_video_post(1, None), creator_profile(), None);

    assert_eq!(row.event_kind, 1);
    assert_eq!(row.identifier, None);
}
