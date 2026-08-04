use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::discovery::event_parsing::video_post_from_event;

fn note(content: &str, hashtags: &[&str]) -> Event {
    let tags = hashtags
        .iter()
        .map(|value| Tag::parse(["t", value]).expect("t tag"))
        .collect::<Vec<_>>();
    EventBuilder::new(Kind::Custom(1), content)
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed note")
}

#[test]
fn event_parsing_merges_t_tags_with_content_hashtags() {
    // Order and normalization mirror nostr_video_event_mapper.dart
    // `_hashtags` + lib/features/video_catalog/domain/video_hashtags.dart:
    // t-tags first (trimmed, lowered, '#' stripped), then content hashtags,
    // deduplicated in first-seen order.
    let post = video_post_from_event(&note(
        "clip https://cdn.example/a.mp4 #Sunset ride #Caf\u{e9}_2026 ## #skate",
        &["#Skate", " RIDE ", " "],
    ))
    .expect("parsed post");

    assert_eq!(post.hashtags, ["skate", "ride", "sunset", "caf\u{e9}_2026"]);
}

#[test]
fn event_parsing_extracts_unicode_content_hashtags() {
    // Dart hashtagPattern `#([\p{L}\p{N}_]+)` is unicode-aware and stops at
    // punctuation.
    let post = video_post_from_event(&note(
        "https://cdn.example/a.mp4 #V\u{ed}deo! mid#word #tag_1,#tag_1",
        &[],
    ))
    .expect("parsed post");

    assert_eq!(post.hashtags, ["v\u{ed}deo", "word", "tag_1"]);
}

#[test]
fn event_parsing_leaves_hashtags_empty_when_none_exist() {
    let post =
        video_post_from_event(&note("plain https://cdn.example/a.mp4", &[])).expect("parsed post");
    assert!(post.hashtags.is_empty());
}
