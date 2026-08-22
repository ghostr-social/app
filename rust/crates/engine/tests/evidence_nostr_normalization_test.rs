use ghostr_engine::catalog::Catalog;
use ghostr_engine::evidence::{EvidenceField, EvidenceValue, NostrMetadataEvidence};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn every_nostr_planning_field_enters_the_typed_ledger_with_real_provenance() {
    let url = "https://media.example/video.mp4";
    let mut catalog = Catalog::new();
    let meta = VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: Some("a".repeat(64)),
        size_bytes: Some(400),
        duration_ms: Some(2_000),
    };
    let evidence = NostrMetadataEvidence {
        issuer: "publisher".into(),
        client: Some("encoder/2".into()),
        event_id: "event".into(),
        observed_at_ms: 10,
        urls: vec![url.into()],
        mime: Some("video/mp4".into()),
        size_bytes: meta.size_bytes,
        duration_ms: meta.duration_ms,
        dimensions: Some((608, 1080)),
        bitrate_bps: Some(900_000),
        sha256: meta.sha256.clone(),
        original_sha256: Some("b".repeat(64)),
    };

    catalog.upsert_with_evidence(PostId::new("post"), meta, Vec::new(), vec![evidence]);
    let fused = catalog
        .lookup(&PostId::new("post"))
        .unwrap()
        .evidence_assessment_for(url, 10);

    assert_eq!(
        fused.value(EvidenceField::Mime),
        Some(&EvidenceValue::Mime("video/mp4".into()))
    );
    assert_eq!(
        fused.value(EvidenceField::Dimensions),
        Some(&EvidenceValue::Dimensions {
            width: 608,
            height: 1080
        })
    );
    assert_eq!(
        fused.value(EvidenceField::Bitrate),
        Some(&EvidenceValue::BitrateBps(900_000))
    );
    assert_eq!(
        fused.value(EvidenceField::OriginalHash),
        Some(&EvidenceValue::OriginalHash("b".repeat(64)))
    );
}
