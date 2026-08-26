use crate::catalog::{Catalog, LearnedFacts};
use crate::evidence::EvidenceField;
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn legacy_http_observation_uses_its_normalized_clock_for_decay() {
    let post = PostId::new("legacy-decay");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), progressive_meta(None, None));
    assert!(catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
    ));

    let entry = catalog.lookup(&post).expect("valid test fixture");
    let source = "https://host.example/video.mp4";
    let assessment = entry.evidence_assessment_for(source, 30 * 24 * 60 * 60 * 1_000);

    assert_eq!(assessment.value(EvidenceField::RangeSupport), None);
    assert!(assessment.stale.contains(&EvidenceField::RangeSupport));
}
