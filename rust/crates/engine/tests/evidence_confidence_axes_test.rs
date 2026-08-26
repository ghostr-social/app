use crate::evidence::{
    Confidence, Evidence, EvidenceLedger, EvidenceScope, EvidenceSource, EvidenceValue,
};

#[test]
fn readiness_never_promotes_byte_identity_and_hash_verification_never_promotes_readiness() {
    let mut ledger = EvidenceLedger::default();
    ledger.record(observation(
        EvidenceValue::Ready(true),
        EvidenceSource::playback("ios"),
    ));
    let readiness_only = ledger.assessment(URL, 10).confidence;
    assert_eq!(readiness_only.readiness, Confidence::certain());
    assert_eq!(readiness_only.integrity, Confidence::none());

    ledger.record(observation(
        EvidenceValue::IntegrityMatch {
            digest: "a".repeat(64),
            matches: true,
        },
        EvidenceSource::hash(URL),
    ));
    let verified = ledger.assessment(URL, 10).confidence;
    assert_eq!(verified.readiness, Confidence::certain());
    assert_eq!(verified.integrity, Confidence::certain());
}

const URL: &str = "https://media.example/video.mp4";

fn observation(value: EvidenceValue, source: EvidenceSource) -> Evidence<EvidenceValue> {
    Evidence::new(
        value,
        source,
        10,
        Confidence::certain(),
        EvidenceScope::url(URL),
    )
}
