use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

#[test]
fn inspected_head_without_timing_requests_only_a_bounded_tail_probe() {
    let mut bench = WorkBench::new();
    let post = PostId::new("tail-moov");
    let binding = bench.catalog.upsert(
        post.clone(),
        progressive_meta(Some(8_000_000), Some(16_000)),
    );
    assert!(bench.catalog.require_tail_timeline_for(&binding));
    bench.focus = focus_at(&["tail-moov"], 0, 0);
    bench
        .present
        .insert(post, vec![ByteRange::new(0, 2_000_000)]);

    let requests = bench.run();

    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].chunk.range,
        ByteRange::new(8_000_000 - 256 * 1_024, 8_000_000)
    );
}

#[test]
fn completed_tail_probe_unlocks_only_one_bounded_reserve_chunk() {
    let mut bench = WorkBench::new();
    bench.params.chunk_bytes = 256 * 1_024;
    let post = PostId::new("tail-moov");
    let binding = bench.catalog.upsert(
        post.clone(),
        progressive_meta(Some(8_000_000), Some(16_000)),
    );
    assert!(bench.catalog.require_tail_timeline_for(&binding));
    bench.focus = focus_at(&["tail-moov"], 0, 0);
    bench.present.insert(
        post,
        vec![
            ByteRange::new(0, 2_000_000),
            ByteRange::new(8_000_000 - 256 * 1_024, 8_000_000),
        ],
    );

    let requests = bench.run();

    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].chunk.range,
        ByteRange::new(2_000_000, 2_000_000 + 256 * 1_024)
    );
}

#[test]
fn optional_timeline_probe_yields_to_an_unstarted_ahead_post() {
    let mut bench = WorkBench::new();
    bench.params.head_seconds = 1;
    bench.params.chunk_bytes = 4;
    let current = PostId::new("current");
    let binding = bench
        .catalog
        .upsert(current.clone(), progressive_meta(Some(80), Some(20_000)));
    assert!(bench.catalog.require_tail_timeline_for(&binding));
    bench.catalog.upsert(
        PostId::new("ahead"),
        progressive_meta(Some(16), Some(4_000)),
    );
    bench.focus = focus_at(&["current", "ahead"], 0, 5_000);
    bench.present.insert(current, vec![ByteRange::new(0, 8)]);

    let requests = bench.run();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].chunk.post, PostId::new("ahead"));
    assert_eq!(requests[0].chunk.range, ByteRange::new(0, 4));
}
