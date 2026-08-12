use ghostr_discovery::content::parsing::video_post_from_event;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};

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
    // Canonical order is normalized t-tags first, then content hashtags,
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
    // Hashtags accept Unicode letters, numbers, and underscores, and stop
    // at punctuation.
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
