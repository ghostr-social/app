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
    let first = log_ids("https://cdn.example/a.mp4");
    let second = log_ids("https://cdn.example/b.mp4");
    let other = log_ids("https://mirror.example/a.mp4");

    assert_eq!(first.0, second.0);
    assert_ne!(first.1, second.1);
    assert_ne!(first.0, other.0);
}

fn log_ids(url: &str) -> (String, String) {
    let rendered = MediaLogIdentity::from_url(url).to_string();
    let values = rendered
        .strip_prefix("media(origin=")
        .and_then(|value| value.strip_suffix(')'))
        .expect("stable media identity format");
    let (origin, object) = values.split_once(", object=").expect("both log identities");
    (origin.to_owned(), object.to_owned())
}
