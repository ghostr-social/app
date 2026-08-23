use ghostr_engine::evidence::{
    Confidence, Evidence, EvidenceField, EvidenceLedger, EvidenceScope, EvidenceSource,
    EvidenceValidator, EvidenceValue,
};

const URL: &str = "https://media.example/video.mp4";

#[test]
fn late_old_validator_cannot_invalidate_newer_generation_evidence() {
    let mut ledger = EvidenceLedger::default();
    let newer = etag("v2");
    assert!(ledger.observe_validator(URL, newer.clone(), 200).is_empty());
    ledger.record(Evidence::new(
        EvidenceValue::FrontMoov(true),
        EvidenceSource::parser("mp4-v3"),
        201,
        Confidence::certain(),
        EvidenceScope::validated(URL, newer),
    ));

    let invalidation = ledger.observe_validator(URL, etag("v1"), 100);

    assert!(invalidation.is_empty());
    assert_eq!(
        ledger.assessment(URL, 202).value(EvidenceField::FrontMoov),
        Some(&EvidenceValue::FrontMoov(true))
    );
}

fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(format!("\"{value}\"")).unwrap()
}
