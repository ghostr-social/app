use ghostr_engine::catalog::{Catalog, PlaybackEvidence};
use ghostr_engine::evidence::Confidence;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn first_frame_updates_readiness_without_promoting_integrity() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec!["https://cdn.example/video.mp4".into()],
            delivery: DeliveryKind::Progressive,
            sha256: Some("a".repeat(64)),
            size_bytes: None,
            duration_ms: None,
        },
    );

    assert!(
        catalog.learn_playback_for(&binding, PlaybackEvidence::new("android-media3", true, 10),)
    );

    let assessment = catalog
        .lookup(&post)
        .unwrap()
        .evidence_assessment_for("https://cdn.example/video.mp4", 10);
    assert!(assessment.confidence.readiness > Confidence::none());
    assert_eq!(assessment.confidence.integrity, Confidence::none());
}
