use crate::catalog::Catalog;
use crate::evidence::{EvidenceField, EvidenceValue};
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn stored_generation_change_invalidates_structural_timeline_evidence() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(Some(1_000), Some(1_000)));
    let moov = classic_moov(&[500], &[100]);
    let timeline = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).unwrap();
    assert!(catalog.learn_timeline_for(&binding, timeline));
    let url = catalog.lookup(&post).unwrap().meta.urls[0].clone();
    assert_eq!(
        catalog
            .lookup(&post)
            .unwrap()
            .evidence_assessment_for(&url, 1)
            .value(EvidenceField::FrontMoov),
        Some(&EvidenceValue::FrontMoov(true))
    );

    assert!(catalog.clear_timeline_for(&binding));

    assert!(catalog.lookup(&post).unwrap().timeline().is_none());
    assert_eq!(
        catalog
            .lookup(&post)
            .unwrap()
            .evidence_assessment_for(&url, 2)
            .value(EvidenceField::FrontMoov),
        None
    );
}
