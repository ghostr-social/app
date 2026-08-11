use crate::media_timeline::{parse_mp4_segments, MediaSegment, PlaybackWindow};
use crate::tests::media_timeline_support::classic_moov;
use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

#[test]
fn current_playback_requests_exact_vbr_ranges_instead_of_an_average_frontier() {
    let mut bench = WorkBench::new();
    bench.params.head_seconds = 1;
    bench.params.chunk_bytes = 2_000;
    let post = PostId::new("current");
    let binding = bench
        .catalog
        .upsert(post.clone(), progressive_meta(Some(20_000), Some(4_000)));
    let moov = classic_moov(&[100, 500, 4_000, 8_000], &[100, 900, 200, 800]);
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
        .insert(post, PlaybackWindow::try_new(0, 2_000).unwrap());

    let requests = bench.run();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].chunk.range, ByteRange::new(500, 1_400));
}

#[test]
fn incomplete_timeline_startup_requests_only_exact_missing_extents() {
    let mut bench = WorkBench::new();
    bench.params.head_seconds = 1;
    let post = PostId::new("current");
    let binding = bench
        .catalog
        .upsert(post.clone(), progressive_meta(Some(20_000), Some(2_000)));
    let moov = classic_moov(&[100, 500], &[100, 900]);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();
    assert!(bench.catalog.learn_timeline_for(&binding, timeline));
    bench.focus = focus_at(&["current"], 0, 0);

    let requests = bench.run();

    assert_eq!(requests[0].chunk.range, ByteRange::new(100, 200));
}

#[test]
fn timeline_without_player_horizon_grants_one_second_of_reserve() {
    let mut bench = WorkBench::new();
    bench.params.head_seconds = 1;
    let post = PostId::new("current");
    let binding = bench
        .catalog
        .upsert(post.clone(), progressive_meta(Some(20_000), Some(3_000)));
    let moov = classic_moov(&[100, 500, 900], &[100, 100, 100]);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();
    assert!(bench.catalog.learn_timeline_for(&binding, timeline));
    bench.focus = focus_at(&["current"], 0, 3_000);
    bench.present.insert(
        post,
        vec![ByteRange::new(100, 200), ByteRange::new(10_000, 20_000)],
    );

    let requests = bench.run();

    assert_eq!(requests[0].chunk.range, ByteRange::new(500, 600));
}
