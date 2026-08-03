use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;

const MAX_NATIVE_TEXT_CHARACTERS: usize = 4_096;

#[test]
fn bounds_untrusted_text_retained_by_the_native_inventory() {
    let oversized = "🦀".repeat(MAX_NATIVE_TEXT_CHARACTERS + 1);
    let tags = [
        Tag::parse(["title".to_owned(), oversized.clone()]).expect("title tag"),
        Tag::parse([
            "imeta".to_owned(),
            "url https://media.example/video.mp4".to_owned(),
            "m video/mp4".to_owned(),
            format!("title {oversized}"),
        ])
        .expect("video tag"),
    ];
    let event = EventBuilder::new(Kind::Custom(22), oversized)
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let videos = canonical_native_videos(&event);

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.content.chars().count(), 4_096);
    assert_eq!(videos[0].video.title.chars().count(), 4_096);
    assert_eq!(videos[0].video.song_name.chars().count(), 4_096);
}

#[test]
fn rejects_unbounded_identifiers_and_media_urls() {
    let oversized_identifier = "d".repeat(513);
    let oversized_url = format!("https://media.example/{}.mp4", "u".repeat(2_048));
    let addressable = EventBuilder::new(Kind::Custom(34236), "clip")
        .tags([
            Tag::parse(["d".to_owned(), oversized_identifier]).expect("identifier"),
            Tag::parse([
                "imeta",
                "url https://media.example/video.mp4",
                "m video/mp4",
            ])
            .expect("video tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("addressable event");
    let oversized_media = EventBuilder::new(Kind::Custom(22), "clip")
        .tag(
            Tag::parse([
                "imeta".to_owned(),
                format!("url {oversized_url}"),
                "m video/mp4".to_owned(),
            ])
            .expect("oversized media tag"),
        )
        .sign_with_keys(&Keys::generate())
        .expect("video event");

    assert!(canonical_native_videos(&addressable).is_empty());
    assert!(canonical_native_videos(&oversized_media).is_empty());
}
