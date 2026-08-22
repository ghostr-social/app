use ghostr_media_model::native_media_metadata::lenient_native_media;

#[test]
fn imeta_preserves_declared_mime_and_separates_x_from_ox() {
    let x = "a".repeat(64);
    let ox = "b".repeat(64);
    let tag = vec![
        "imeta".to_owned(),
        "url https://media.example/video.mp4".to_owned(),
        "m video/mp4".to_owned(),
        format!("x {x}"),
        format!("ox {ox}"),
    ];

    let media = lenient_native_media(&tag).unwrap();

    assert_eq!(media.declared_mime.as_deref(), Some("video/mp4"));
    assert_eq!(media.expected_digest.as_deref(), Some(x.as_str()));
    assert_eq!(media.original_digest.as_deref(), Some(ox.as_str()));
}
