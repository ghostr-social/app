use ghostr_engine::evidence::{
    Confidence, Evidence, EvidenceField, EvidenceLedger, EvidenceScope, EvidenceSource,
    EvidenceValue,
};

#[test]
fn expired_network_evidence_is_stale_instead_of_missing_or_actionable() {
    let url = "https://cdn.example/video.mp4";
    let mut ledger = EvidenceLedger::default();
    ledger.record(Evidence::new(
        EvidenceValue::RangeSupport(true),
        EvidenceSource::head("cdn.example"),
        1,
        Confidence::new(8_000).unwrap(),
        EvidenceScope::url(url),
    ));

    let assessment = ledger.assessment(url, 48 * 60 * 60 * 1_000);

    assert!(assessment.stale.contains(&EvidenceField::RangeSupport));
    assert!(!assessment.missing.contains(&EvidenceField::RangeSupport));
    assert_eq!(assessment.value(EvidenceField::RangeSupport), None);
}
