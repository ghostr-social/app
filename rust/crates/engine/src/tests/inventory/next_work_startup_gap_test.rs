use crate::playback::PLAYBACK_SLICE_BYTES;
use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

const CACHED_BLOCK: u64 = 64 * 1_024;
const TOTAL: u64 = 370_912;

#[test]
fn a_tail_probe_does_not_count_as_startup_depth() {
    let mut bench = WorkBench::new();
    for post in ["current", "tail-only", "deeper"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(TOTAL), Some(3_000)),
        );
    }
    bench.focus = focus_at(&["current", "tail-only", "deeper"], 0, 0);
    bench
        .present
        .insert(PostId::new("current"), vec![ByteRange::new(0, TOTAL)]);
    bench.present.insert(
        PostId::new("tail-only"),
        vec![ByteRange::new(300_000, TOTAL)],
    );
    bench
        .present
        .insert(PostId::new("deeper"), vec![ByteRange::new(0, CACHED_BLOCK)]);

    let first = bench.run().remove(0);

    assert_eq!(first.chunk.post, PostId::new("tail-only"));
    assert_eq!(first.chunk.range, ByteRange::new(0, PLAYBACK_SLICE_BYTES));
}

#[test]
fn filling_a_prefix_gap_counts_the_cached_bytes_it_connects() {
    let mut bench = WorkBench::new();
    for post in ["current", "bridged", "peer"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(TOTAL), Some(3_000)),
        );
    }
    bench.focus = focus_at(&["current", "bridged", "peer"], 0, 0);
    bench
        .present
        .insert(PostId::new("current"), vec![ByteRange::new(0, TOTAL)]);
    bench.present.insert(
        PostId::new("bridged"),
        vec![
            ByteRange::new(0, CACHED_BLOCK),
            ByteRange::new(2 * CACHED_BLOCK, 3 * CACHED_BLOCK),
        ],
    );
    bench.present.insert(
        PostId::new("peer"),
        vec![ByteRange::new(0, 2 * CACHED_BLOCK)],
    );

    let first_two: Vec<_> = bench
        .run()
        .into_iter()
        .take(2)
        .map(|work| work.chunk.post)
        .collect();

    assert_eq!(first_two, [PostId::new("bridged"), PostId::new("peer")]);
}
