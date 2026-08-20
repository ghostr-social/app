use ghostr_engine::catalog::Catalog;
use ghostr_engine::evidence::{
    CalibrationContext, CalibrationDimensions, EvidenceField, NostrMetadataEvidence,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn complete_bytes_label_the_exact_issuer_origin_url_and_size_context() {
    let url = "https://cdn.example/video.mp4";
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let meta = VideoMeta {
        urls: vec![url.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(10),
        duration_ms: None,
    };
    let declared = NostrMetadataEvidence {
        issuer: "issuer".into(),
        event_id: "event".into(),
        observed_at_ms: 1,
        urls: vec![url.into()],
        size_bytes: Some(10),
        ..Default::default()
    };
    let binding = catalog.upsert_with_evidence(post, meta, Vec::new(), vec![declared]);
    let identity = binding.transfer(url).unwrap();

    assert!(catalog.learn_complete_bytes_for(&identity, 20, 10));

    let dimensions = CalibrationDimensions::new(
        Some("issuer".into()),
        Some("cdn.example".into()),
        Some(url.into()),
    );
    let context = CalibrationContext::new(dimensions, EvidenceField::Size, "nostr");
    assert!(catalog.field_reliability().estimate(&context, 10).mean_bps < 5_000);
}
