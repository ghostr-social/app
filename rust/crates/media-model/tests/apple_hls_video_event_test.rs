use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use ghostr_media_model::event_identity::canonical_native_videos;
use ghostr_media_model::native_models::NativeVideoDelivery;

#[test]
fn accepts_the_registered_apple_hls_media_type() {
    let event = EventBuilder::new(Kind::Custom(34236), "Apple HLS stream")
        .tags([
            Tag::parse(["d", "apple-hls"]).expect("identifier"),
            Tag::parse([
                "imeta",
                "url https://media.example/video.m3u8",
                "m application/vnd.apple.mpegurl",
            ])
            .expect("video tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let videos = canonical_native_videos(&event);

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].video.delivery, NativeVideoDelivery::Hls);
}
