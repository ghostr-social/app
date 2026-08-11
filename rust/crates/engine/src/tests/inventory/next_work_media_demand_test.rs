use crate::media_timeline::{parse_mp4_segments, MediaSegment, PlaybackWindow};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

#[test]
fn decoder_demand_stays_an_exact_range_when_timing_is_available() {
    let mut bench = WorkBench::new();
    bench.params.head_seconds = 1;
    let post = PostId::new("current");
    let binding = bench
        .catalog
        .upsert(post.clone(), progressive_meta(Some(20_000), Some(2_000)));
    let moov = classic_moov(&[100, 500], &[100, 900]);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();
    assert!(bench.catalog.learn_timeline_for(&binding, timeline));
    bench.focus = focus_at(&["current"], 0, 3_000);
    bench.present.insert(
        post.clone(),
        vec![
            ByteRange::new(100, 200),
            ByteRange::new(10_000, 10_000 + moov.len() as u64),
        ],
    );
    bench
        .media_window
        .insert(post.clone(), PlaybackWindow::try_new(0, 1_000).unwrap());
    bench
        .direct_range
        .insert(post, ByteRange::new(9_000, 9_100));

    let requests = bench.run();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].chunk.range, ByteRange::new(9_000, 9_100));
}
