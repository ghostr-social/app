use crate::catalog::Catalog;
use crate::evidence::{EvidenceField, EvidenceValue};
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn parsing_one_mirror_does_not_certify_another_mirrors_layout() {
    let post = PostId::new("post");
    let mut meta = progressive_meta(Some(1_000), Some(1_000));
    let source = meta.urls[0].clone();
    let mirror = "https://other.example/video.mp4";
    meta.urls.push(mirror.to_owned());
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), meta);
    let moov = classic_moov(&[500], &[100]);
    let timeline = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).expect("fixture");
    assert!(catalog.learn_timeline_for(&binding, timeline));
    let entry = catalog.lookup(&post).expect("fixture");
    assert_eq!(
        entry
            .evidence_assessment_for(&source, 1)
            .value(EvidenceField::FrontMoov),
        Some(&EvidenceValue::FrontMoov(true))
    );
    assert_eq!(
        entry
            .evidence_assessment_for(mirror, 1)
            .value(EvidenceField::FrontMoov),
        None
    );
}
