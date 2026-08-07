use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use ghostr_media_model::event_identity::canonical_native_videos;

#[test]
fn accepts_a_nip71_hls_playlist() {
    let event = EventBuilder::new(Kind::Custom(34236), "Adaptive relay video")
        .tags([
            Tag::parse(["d", "hls-clip"]).expect("identifier"),
            Tag::parse([
                "imeta",
                "url https://media.example/video.m3u8",
                "m application/x-mpegURL",
            ])
            .expect("video tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let videos = canonical_native_videos(&event);

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].video.url, "https://media.example/video.m3u8");
}
