use ghostr_engine::catalog::{Catalog, LearnedFacts};
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

#[test]
fn range_response_labels_the_advisory_head_claim() {
    let url = "https://cdn.example/video.mp4";
    let post = PostId::new("range-post");
    let mut catalog = Catalog::new();
    let meta = VideoMeta {
        urls: vec![url.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    };
    let binding = catalog.upsert(post, meta);
    let identity = binding.transfer(url).unwrap();
    assert!(catalog.learn_head_for(&identity, range_facts(true)));
    assert!(catalog.learn_response_for(&identity, range_facts(false)));
    let dimensions = CalibrationDimensions::new(None, Some("cdn.example".into()), Some(url.into()));
    let context = CalibrationContext::new(dimensions, EvidenceField::RangeSupport, "head");

    let estimate = catalog.field_reliability().estimate(&context, 2);

    assert!(estimate.mean_bps < 5_000);
    assert!(estimate.effective_samples_bps > 0);
}

fn range_facts(accept_ranges: bool) -> LearnedFacts {
    LearnedFacts {
        accept_ranges: Some(accept_ranges),
        ..LearnedFacts::default()
    }
}
