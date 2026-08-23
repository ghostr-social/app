use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::{EvidenceField, EvidenceValidator, EvidenceValue};
use crate::tests::support::progressive_meta;
use crate::PostId;

const SOURCE: &str = "https://host.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn strong_validator_does_not_permanently_pin_range_behavior() {
    let post = PostId::new("range-recovery");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        post.clone(),
        progressive_meta(Some(2_000_000), Some(10_000)),
    );
    let identity = binding.transfer(SOURCE).expect("source identity");
    let observation = HttpObservation::new(
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
        None,
        OBSERVED_AT_MS,
        EvidenceValidator::strong_etag("\"generation-1\""),
    );
    assert!(catalog.learn_response_observation_for(&identity, observation));
    let entry = catalog.lookup(&post).expect("catalog entry");

    assert_eq!(
        entry
            .evidence_assessment_for(SOURCE, OBSERVED_AT_MS)
            .value(EvidenceField::RangeSupport),
        Some(&EvidenceValue::RangeSupport(false))
    );
    let stale = entry.evidence_assessment_for(SOURCE, OBSERVED_AT_MS + DAY_MS);
    assert_eq!(stale.value(EvidenceField::RangeSupport), None);
    assert!(stale.stale.contains(&EvidenceField::RangeSupport));
}
