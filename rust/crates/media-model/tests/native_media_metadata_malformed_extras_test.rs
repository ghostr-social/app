use ghostr_media_model::imeta_extras::ImetaExtras;
use ghostr_media_model::native_media_metadata::native_media;

fn with_field(field: &str) -> Vec<String> {
    vec![
        "imeta".to_owned(),
        "url https://cdn.example/clip.mp4".to_owned(),
        "m video/mp4".to_owned(),
        field.to_owned(),
    ]
}

#[test]
fn native_media_metadata_turns_malformed_extras_into_none_without_failing() {
    // Lenient like VideoMediaMetadata.fromImeta in
    // lib/core/media/video_media_metadata.dart: a malformed field becomes
    // null while the media itself stays playable.
    let cases: &[&str] = &[
        "size zero",
        "size 0",
        "size -20",
        "size 12.5",
        "duration soon",
        "duration -3",
        "duration 0",
        "duration Infinity",
        "dim 1920",
        "dim x1080",
        "dim 0x1080",
        "dim 1920x",
        "dim 1920xtall",
        "image ftp://cdn.example/thumb.jpg",
        "image notaurl",
    ];
    for field in cases {
        let media = native_media(&with_field(field)).expect(field);
        assert_eq!(media.extras, ImetaExtras::default(), "{field}");
        assert_eq!(media.url, "https://cdn.example/clip.mp4", "{field}");
    }
}

#[test]
fn native_media_metadata_leaves_absent_extras_as_none() {
    let media = native_media(&[
        "imeta".to_owned(),
        "url https://cdn.example/clip.mp4".to_owned(),
        "m video/mp4".to_owned(),
    ])
    .expect("media");
    assert_eq!(media.extras, ImetaExtras::default());
}
