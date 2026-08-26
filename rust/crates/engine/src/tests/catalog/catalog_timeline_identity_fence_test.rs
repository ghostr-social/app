use crate::catalog::Catalog;
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn evicted_representation_cannot_restore_timing_state() {
    let post = PostId::new("evicted");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(Some(1_000), Some(1_000)));
    let moov = classic_moov(&[100], &[100]);
    let timeline =
        parse_mp4_segments(&[MediaSegment::new(500, &moov)]).expect("valid test fixture");
    catalog.retain(|known| known != &post);

    assert!(!catalog.learn_timeline_for(&binding, timeline));
    assert!(!catalog.require_tail_timeline_for(&binding));
}
