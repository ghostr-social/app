use crate::representation::RepresentationId;
use crate::{DeliveryKind, VideoMeta};

#[test]
fn representation_identity_matches_the_flutter_contract() {
    assert_eq!(
        RepresentationId::from_meta(&advertised()).fingerprint(),
        "40b13ee390bb98651d749f074546e825c163cc0886d7a5ce51210cbbf6e761da"
    );
    assert_eq!(
        RepresentationId::from_meta(&unverified()).fingerprint(),
        "ede223d32401527e82b2e523f2a5ede1837019d2b46551995acefb2e2b0b70ea"
    );
    assert_eq!(
        RepresentationId::from_meta(&unicode_ordered()).fingerprint(),
        "2ec09b10cf5150d379803130724d865cbb743b4c32048a6b3f25431cd56c5d51"
    );
}

fn unicode_ordered() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://media.test/\u{10000}.mp4".to_owned(),
            "https://media.test/\u{e000}.mp4".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}

fn advertised() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://ignored.test/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: Some("a".repeat(64)),
        size_bytes: None,
        duration_ms: None,
    }
}

fn unverified() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://b.test/video.mp4".to_owned(),
            "https://a.test/video.mp4".to_owned(),
            "https://b.test/video.mp4".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(123_456),
        duration_ms: Some(9_876),
    }
}
