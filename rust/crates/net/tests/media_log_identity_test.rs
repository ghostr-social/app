use ghostr_net::media_log_identity::MediaLogIdentity;

#[test]
fn media_log_identity_hides_url_credentials_path_and_query() {
    let raw = "https://alice:secret@cdn.example/private/video.mp4?token=sensitive";
    let rendered = MediaLogIdentity::from_url(raw).to_string();

    for secret in [
        "alice",
        "secret",
        "cdn.example",
        "private",
        "video.mp4",
        "sensitive",
    ] {
        assert!(!rendered.contains(secret), "leaked {secret}: {rendered}");
    }
    assert!(rendered.starts_with("media(origin="));
    assert!(rendered.contains(", object="));
}

#[test]
fn media_log_identity_separates_origin_and_object_correlation() {
    let first = MediaLogIdentity::from_url("https://cdn.example/a.mp4");
    let second = MediaLogIdentity::from_url("https://cdn.example/b.mp4");
    let other = MediaLogIdentity::from_url("https://mirror.example/a.mp4");

    assert_eq!(first.origin_id(), second.origin_id());
    assert_ne!(first.object_id(), second.object_id());
    assert_ne!(first.origin_id(), other.origin_id());
}
