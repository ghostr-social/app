//! Publishers pad tag values. Mime matching is whitespace- and
//! ASCII-case-insensitive, so padding must not cost an otherwise playable
//! post.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use ghostr_discovery::content::parsing::video_post_from_event;
use ghostr_engine::DeliveryKind;

fn video_event(mime: &str) -> Event {
    EventBuilder::new(Kind::Custom(22), "short clip")
        .tags(vec![
            Tag::parse(["url", "https://cdn.example/clip.mp4"]).expect("url tag"),
            Tag::parse(["m", mime]).expect("m tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

#[test]
fn event_parsing_accepts_a_padded_video_mime() {
    for mime in [" video/mp4", "video/mp4 ", "\tvideo/mp4", " VIDEO/MP4 "] {
        let post = video_post_from_event(&video_event(mime));
        assert!(post.is_some(), "{mime:?} should still be a video");
    }
}

#[test]
fn event_parsing_reads_hls_through_a_padded_mime() {
    let post = video_post_from_event(&video_event(" application/x-mpegurl ")).expect("parsed post");

    assert_eq!(post.meta.delivery, DeliveryKind::Hls);
}

#[test]
fn event_parsing_still_rejects_a_non_video_mime() {
    assert!(video_post_from_event(&video_event(" image/png ")).is_none());
}
