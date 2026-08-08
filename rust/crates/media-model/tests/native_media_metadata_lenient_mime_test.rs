use ghostr_media_model::native_media_metadata::lenient_native_media;
use ghostr_media_model::native_models::NativeVideoDelivery;

fn imeta(fields: &[&str]) -> Vec<String> {
    let mut tag = vec!["imeta".to_owned()];
    tag.extend(fields.iter().map(|field| (*field).to_owned()));
    tag
}

#[test]
fn native_media_metadata_accepts_mimeless_imeta_with_a_video_extension() {
    // Publishers often omit the mime; recognized URL extensions still
    // identify playable media.
    let cases = [
        (
            "https://cdn.example/a.mp4",
            Some(NativeVideoDelivery::Progressive),
        ),
        (
            "https://cdn.example/a.m4v",
            Some(NativeVideoDelivery::Progressive),
        ),
        (
            "https://cdn.example/a.webm",
            Some(NativeVideoDelivery::Progressive),
        ),
        (
            "https://cdn.example/A.MOV",
            Some(NativeVideoDelivery::Progressive),
        ),
        (
            "https://cdn.example/live.m3u8",
            Some(NativeVideoDelivery::Hls),
        ),
        ("https://cdn.example/photo.jpg", None),
        ("https://cdn.example/clip", None),
    ];
    for (url, expected) in cases {
        let media = lenient_native_media(&imeta(&[&format!("url {url}")]));
        assert_eq!(
            media.as_ref().map(|media| media.delivery),
            expected,
            "{url}"
        );
        if let Some(media) = media {
            assert_eq!(media.url, url, "{url}");
        }
    }
}

#[test]
fn native_media_metadata_still_honors_an_explicit_mime_when_lenient() {
    let hls = lenient_native_media(&imeta(&[
        "url https://cdn.example/stream",
        "m application/vnd.apple.mpegurl",
    ]))
    .expect("hls media");
    assert_eq!(hls.delivery, NativeVideoDelivery::Hls);

    let image = lenient_native_media(&imeta(&["url https://cdn.example/clip.mp4", "m image/png"]));
    assert!(image.is_none(), "non-video mime still rejects the tag");
}

#[test]
fn native_media_metadata_rejects_an_imeta_with_no_usable_url() {
    // An imeta tag without a bounded HTTP URL cannot produce media.
    let none = lenient_native_media(&imeta(&["m video/mp4", "url notaurl"]));

    assert!(none.is_none());
}

#[test]
fn native_media_metadata_uses_a_fallback_when_the_primary_is_unusable() {
    // Primary and fallback URLs form one ordered set, so a broken primary
    // still leaves the fallbacks playable.
    let media = lenient_native_media(&imeta(&[
        "url notaurl",
        "fallback https://mirror.example/clip.mp4",
        "fallback https://mirror.example/clip.mp4",
        "fallback https://other.example/clip.mp4",
    ]))
    .expect("fallback media");
    assert_eq!(media.url, "https://mirror.example/clip.mp4");
    assert_eq!(media.fallback_urls, ["https://other.example/clip.mp4"]);
}
