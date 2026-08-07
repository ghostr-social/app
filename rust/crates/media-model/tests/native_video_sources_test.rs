mod media_fixture;

use ghostr_media_model::native_models::NativeVideoCacheKey;
use media_fixture::native_video;

#[test]
fn native_video_source_identity_uses_digest_or_url_namespace() {
    let mut advertised = native_video("https://primary.example/clip.mp4");
    advertised.expected_digest = Some("c".repeat(64));
    advertised.fallback_urls = vec![
        "https://mirror-one.example/clip.mp4".to_owned(),
        "https://mirror-two.example/clip.mp4".to_owned(),
    ];

    assert_eq!(
        advertised.cache_key(),
        NativeVideoCacheKey::AdvertisedDigest("c".repeat(64))
    );
    assert_eq!(
        advertised.source_urls().collect::<Vec<_>>(),
        [
            "https://primary.example/clip.mp4",
            "https://mirror-one.example/clip.mp4",
            "https://mirror-two.example/clip.mp4",
        ]
    );

    let hashless = native_video("https://primary.example/hashless.mp4");
    assert_eq!(
        hashless.cache_key(),
        NativeVideoCacheKey::UrlDerived(hashless.id.clone())
    );
}
