use ghostr_engine::evidence::{
    Confidence, Evidence, EvidenceField, EvidenceLedger, EvidenceScope, EvidenceSource,
    EvidenceValidator, EvidenceValue,
};

#[test]
fn validator_change_invalidates_url_structural_evidence_but_retains_its_history() {
    let mut ledger = EvidenceLedger::default();
    let first = etag("v1");
    let second = etag("v2");
    assert!(ledger.observe_validator(URL, first.clone(), 10).is_empty());
    ledger.record(Evidence::new(
        EvidenceValue::FrontMoov(true),
        EvidenceSource::parser("mp4-v3"),
        11,
        Confidence::certain(),
        EvidenceScope::validated(URL, first),
    ));
    assert_eq!(
        ledger.assessment(URL, 12).value(EvidenceField::FrontMoov),
        Some(&EvidenceValue::FrontMoov(true))
    );

    let invalidation = ledger.observe_validator(URL, second, 20);
    assert_eq!(invalidation.invalidated_records, 1);
    assert!(invalidation.structural_evidence);
    assert_eq!(
        ledger.assessment(URL, 20).value(EvidenceField::FrontMoov),
        None
    );
    assert_eq!(
        ledger.records().len(),
        1,
        "invalid evidence remains auditable"
    );
    assert!(ledger.records()[0].invalidated_at_ms().is_some());
}

#[test]
fn first_validator_invalidates_provisional_url_structural_evidence() {
    let mut ledger = EvidenceLedger::default();
    ledger.record(Evidence::new(
        EvidenceValue::FrontMoov(true),
        EvidenceSource::parser("mp4-v3"),
        10,
        Confidence::certain(),
        EvidenceScope::url(URL),
    ));

    let invalidation = ledger.observe_validator(URL, etag("v1"), 20);

    assert_eq!(invalidation.invalidated_records, 1);
    assert!(invalidation.structural_evidence);
    assert_eq!(
        ledger.assessment(URL, 20).value(EvidenceField::FrontMoov),
        None
    );
}

const URL: &str = "https://media.example/video.mp4";

fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(format!("\"{value}\"")).unwrap()
}
