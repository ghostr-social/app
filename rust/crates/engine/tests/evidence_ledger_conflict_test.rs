use crate::evidence::{
    Confidence, Evidence, EvidenceLedger, EvidenceScope, EvidenceSource, EvidenceValidator,
    EvidenceValue,
};

#[test]
fn contradictory_sizes_remain_visible_until_direct_bytes_resolve_the_bound() {
    let mut ledger = EvidenceLedger::default();
    ledger.record(size(
        800,
        EvidenceSource::nostr("issuer"),
        EvidenceScope::url(URL),
        10,
    ));
    ledger.record(size(
        1_200,
        EvidenceSource::head(URL),
        EvidenceScope::url(URL),
        20,
    ));

    let conflicted = ledger.assessment(URL, 20);
    assert_eq!(conflicted.size.lower, Some(800));
    assert_eq!(conflicted.size.upper, Some(1_200));
    assert!(conflicted.size.conflict);
    assert!(!conflicted.size.reliable);

    ledger.record(size(
        1_000,
        EvidenceSource::CompleteBytes {
            origin: URL.to_owned(),
        },
        EvidenceScope::validated(URL, etag("v1")),
        30,
    ));
    let resolved = ledger.assessment(URL, 30);
    assert_eq!(resolved.size.exact, Some(1_000));
    assert!(resolved.size.conflict);
    assert!(resolved.size.resolved_by_direct_evidence);
    assert!(resolved.size.reliable);
    assert_eq!(ledger.records().len(), 3);
}

const URL: &str = "https://media.example/video.mp4";

fn size(
    bytes: u64,
    source: EvidenceSource,
    scope: EvidenceScope,
    observed_at_ms: u64,
) -> Evidence<EvidenceValue> {
    Evidence::new(
        EvidenceValue::SizeBytes(bytes),
        source,
        observed_at_ms,
        Confidence::new(9_000).expect("valid test fixture"),
        scope,
    )
}

fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(format!("\"{value}\"")).expect("valid test fixture")
}
