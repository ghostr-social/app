use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use ghostr_discovery::event_parsing::video_post_from_event;
use ghostr_engine::DeliveryKind;

fn file_event(tags: Vec<Tag>) -> Event {
    EventBuilder::new(Kind::Custom(1063), "file description")
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

#[test]
fn event_parsing_maps_nip94_top_level_file_tags() {
    // NIP-94 carries URL, mime, and digest metadata as top-level tags.
    let post = video_post_from_event(&file_event(vec![
        Tag::parse(["url", "https://file.example/clip.mp4"]).expect("url tag"),
        Tag::parse(["m", "video/mp4"]).expect("m tag"),
        Tag::parse(["x", &"C".repeat(64)]).expect("x tag"),
    ]))
    .expect("parsed post");

    assert_eq!(post.meta.urls, ["https://file.example/clip.mp4"]);
    assert_eq!(post.meta.delivery, DeliveryKind::Progressive);
    assert_eq!(post.meta.sha256, Some("c".repeat(64)));
    assert_eq!(post.meta.size_bytes, None);
    assert_eq!(post.meta.duration_ms, None);
    assert_eq!(post.caption, "file description");
    assert_eq!(post.kind, 1063);
}

#[test]
fn event_parsing_uses_the_file_mime_for_hls_delivery() {
    let post = video_post_from_event(&file_event(vec![
        Tag::parse(["url", "https://file.example/stream"]).expect("url tag"),
        Tag::parse(["m", "application/vnd.apple.mpegurl"]).expect("m tag"),
    ]))
    .expect("parsed post");
    assert_eq!(post.meta.delivery, DeliveryKind::Hls);
}

#[test]
fn event_parsing_rejects_file_tags_with_an_invalid_digest() {
    // A present but invalid digest rejects the file metadata; without a
    // text link, the event yields no playable post.
    let post = video_post_from_event(&file_event(vec![
        Tag::parse(["url", "https://file.example/clip.mp4"]).expect("url tag"),
        Tag::parse(["m", "video/mp4"]).expect("m tag"),
        Tag::parse(["x", "nothex"]).expect("x tag"),
    ]));
    assert!(post.is_none());
}
