//! Signed CDN URLs get long. The safety bound must admit realistic links
//! while still keeping pathological content out of the native cache.

use nostr_sdk::{Event, EventBuilder, Keys};
use ghostr_discovery::content::parsing::video_post_from_event;

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
