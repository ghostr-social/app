use ghostr_media_model::native_media_metadata::native_media;

fn imeta(fields: &[&str]) -> Vec<String> {
    let mut tag = vec!["imeta".to_owned()];
    tag.extend(fields.iter().map(|field| (*field).to_owned()));
    tag
}

fn base(extra: &str) -> Vec<String> {
    imeta(&["url https://cdn.example/clip.mp4", "m video/mp4", extra])
}

#[test]
fn native_media_metadata_parses_the_free_imeta_planning_extras() {
    let media = native_media(&imeta(&[
        "url https://cdn.example/clip.mp4",
        "m video/mp4",
        "size 2048",
        "duration 12.5",
        "dim 1080x1920",
        "blurhash LEHV6nWB2yk8pyo0adR*.7kCMdnj",
        "image https://cdn.example/thumb.jpg",
    ]))
    .expect("media");

    assert_eq!(media.extras.size_bytes, Some(2048));
    assert_eq!(media.extras.duration_ms, Some(12_500));
    assert_eq!(media.extras.dimensions, Some((1080, 1920)));
    assert_eq!(
        media.extras.blurhash.as_deref(),
        Some("LEHV6nWB2yk8pyo0adR*.7kCMdnj")
    );
    assert_eq!(
        media.extras.image_url.as_deref(),
        Some("https://cdn.example/thumb.jpg")
    );
}

#[test]
fn native_media_metadata_converts_imeta_duration_seconds_to_milliseconds() {
    // Units mirror lib/core/media/video_media_metadata.dart: imeta carries
    // seconds (fractional allowed), delivery planning wants milliseconds.
    let cases = [
        ("duration 12", 12_000),
        ("duration 12.5", 12_500),
        ("duration 0.25", 250),
        ("duration 1e1", 10_000),
        ("duration  7 ", 7_000),
    ];
    for (field, expected) in cases {
        let media = native_media(&base(field)).expect(field);
        assert_eq!(media.extras.duration_ms, Some(expected), "{field}");
    }
}

#[test]
fn native_media_metadata_reads_imeta_size_as_bytes() {
    // lib/core/media/video_media_metadata.dart: `size` is bytes, > 0.
    let media = native_media(&base("size 123456789")).expect("media");
    assert_eq!(media.extras.size_bytes, Some(123_456_789));
}
