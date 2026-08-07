use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use ghostr_discovery::event_parsing::video_post_from_event;
use ghostr_engine::DeliveryKind;

#[test]
fn event_parsing_maps_an_imeta_event_into_a_parsed_post() {
    let event = EventBuilder::new(
        Kind::Custom(22),
        "Skate clip https://cdn.example/a.mp4 #Skate day",
    )
    .tags([
        Tag::parse(["t", "Sunset"]).expect("t tag"),
        Tag::parse([
            "imeta".to_owned(),
            "url https://cdn.example/a.mp4".to_owned(),
            "m video/mp4".to_owned(),
            format!("x {}", "A".repeat(64)),
            "fallback https://mirror.example/a.mp4".to_owned(),
            "fallback https://cdn.example/a.mp4".to_owned(),
            "size 2048".to_owned(),
            "duration 12.5".to_owned(),
            "dim 1080x1920".to_owned(),
            "blurhash LKO2?U%2Tw=w".to_owned(),
            "image https://cdn.example/a.jpg".to_owned(),
            "title Night skate".to_owned(),
        ])
        .expect("imeta tag"),
    ])
    .sign_with_keys(&Keys::generate())
    .expect("signed event");

    let post = video_post_from_event(&event).expect("parsed post");

    // URLs keep primary-then-fallback order with deduplication, digests
    // are lowercased, and playable URLs are removed from the caption.
    assert_eq!(
        post.meta.urls,
        ["https://cdn.example/a.mp4", "https://mirror.example/a.mp4"]
    );
    assert_eq!(post.meta.delivery, DeliveryKind::Progressive);
    assert_eq!(post.meta.sha256, Some("a".repeat(64)));
    assert_eq!(post.meta.size_bytes, Some(2048));
    assert_eq!(post.meta.duration_ms, Some(12_500));
    assert_eq!(post.caption, "Skate clip #Skate day");
    assert_eq!(post.title.as_deref(), Some("Night skate"));
    assert_eq!(post.hashtags, ["sunset", "skate"]);
    assert_eq!(post.dimensions, Some((1080, 1920)));
    assert_eq!(post.blurhash.as_deref(), Some("LKO2?U%2Tw=w"));
    assert_eq!(
        post.thumbnail_url.as_deref(),
        Some("https://cdn.example/a.jpg")
    );
    assert_eq!(post.event_id, event.id.to_hex());
    assert_eq!(post.author_pubkey, event.pubkey.to_hex());
    assert_eq!(post.created_at, event.created_at.as_u64());
    assert_eq!(post.kind, 22);
    assert_eq!(post.identifier, None);
}

#[test]
fn event_parsing_keeps_the_identifier_of_addressable_posts() {
    let event = EventBuilder::new(Kind::Custom(34235), "clip")
        .tags([
            Tag::parse(["d", " portrait "]).expect("d tag"),
            Tag::parse(["imeta", "url https://cdn.example/a.mp4", "m video/mp4"])
                .expect("imeta tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let post = video_post_from_event(&event).expect("parsed post");
    // Identifier is trimmed like NostrEventIdentifier.parse in
    // lib/core/nostr/nostr_event_identity.dart.
    assert_eq!(post.identifier.as_deref(), Some("portrait"));
    assert_eq!(post.kind, 34235);
}
