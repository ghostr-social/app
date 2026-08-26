use crate::content::parsing::video_post_from_event;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

fn imeta(fields: &[&str]) -> Tag {
    let mut values = vec!["imeta"];
    values.extend_from_slice(fields);
    Tag::parse(values).expect("imeta tag")
}

#[test]
fn nip71_repeated_imeta_keeps_progressive_renditions_and_their_own_mirrors() {
    let event = EventBuilder::new(Kind::Custom(22), "adaptive clip")
        .tags([
            imeta(&[
                "url https://stream.example/video.m3u8",
                "m application/x-mpegURL",
                "bitrate 800000",
            ]),
            imeta(&[
                "url https://high.example/video.mp4",
                "fallback https://high-mirror.example/video.mp4",
                "m video/mp4",
                "bitrate 6000000",
                "duration 12",
            ]),
            imeta(&[
                "url https://low.example/video.mp4",
                "fallback https://low-mirror.example/video.mp4",
                "m video/mp4",
                "bitrate 1000000",
                "duration 12",
            ]),
            imeta(&[
                "url https://audio.example/audio.mp3",
                "m audio/mp3",
                "bitrate 128000",
            ]),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let post = video_post_from_event(&event).expect("parsed post");

    assert_eq!(post.meta.urls[0], "https://high.example/video.mp4");
    assert_eq!(post.renditions.len(), 2);
    assert_eq!(
        post.renditions[0].bitrate_bits_per_second(),
        Some(6_000_000)
    );
    assert_eq!(
        post.renditions[0].meta().urls,
        [
            "https://high.example/video.mp4",
            "https://high-mirror.example/video.mp4",
        ]
    );
    assert_eq!(
        post.renditions[1].bitrate_bits_per_second(),
        Some(1_000_000)
    );
    assert_eq!(
        post.renditions[1].meta().urls,
        [
            "https://low.example/video.mp4",
            "https://low-mirror.example/video.mp4",
        ]
    );
}
