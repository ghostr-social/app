use crate::catalog::Catalog;
use crate::inventory_controller::is_startable;
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams, PostId};

#[test]
fn timeline_startability_fails_when_any_required_extent_is_missing() {
    let post = PostId::new("vbr");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(Some(20_000), Some(2_000)));
    let moov = classic_moov(&[100, 500], &[100, 100]);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();
    assert!(catalog.learn_timeline_for(&binding, timeline));
    let have = [
        ByteRange::new(100, 200),
        ByteRange::new(10_000, 10_000 + moov.len() as u64),
    ];

    assert!(!is_startable(
        &catalog,
        &post,
        &have,
        &EngineParams::default()
    ));
}
