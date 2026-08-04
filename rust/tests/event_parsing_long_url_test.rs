//! Signed CDN URLs get long. Dart bounds nothing
//! (lib/features/video_catalog/data/nostr_video_media.dart), so a Rust
//! byte bound tight enough to reject a real link is a parity break that
//! costs the feed a post. The bound stays — pathological content must
//! not reach the native cache — but it sits well past any real URL.

use nostr_sdk::{Event, EventBuilder, Keys};
use rust_lib_ghostr::discovery::event_parsing::video_post_from_event;

fn note_linking(url: &str) -> Event {
    EventBuilder::text_note(format!("watch this {url}"))
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

fn signed_url(padding: usize) -> String {
    format!("https://cdn.example/{}/clip.mp4", "s".repeat(padding))
}

#[test]
fn event_parsing_keeps_a_long_signed_video_url() {
    let url = signed_url(2_100);

    let post = video_post_from_event(&note_linking(&url)).expect("parsed post");

    assert_eq!(post.meta.urls, [url]);
}

#[test]
fn event_parsing_still_refuses_an_absurd_url() {
    let url = signed_url(100_000);

    assert!(video_post_from_event(&note_linking(&url)).is_none());
}
