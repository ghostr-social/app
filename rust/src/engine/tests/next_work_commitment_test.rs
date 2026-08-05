use crate::engine::tests::scheduling_support::{focus_at, WorkBench};
use crate::engine::tests::support::progressive_meta;
use crate::engine::tiers::Tier;
use crate::engine::{ByteRange, PostId};

// Current post: 8 MB / 16 s, head (2 MB) on disk; next post unfetched.
fn bench(watch_ms: u64) -> WorkBench {
    let mut bench = WorkBench::new();
    bench.catalog.upsert(
        PostId::new("a"),
        progressive_meta(Some(8_000_000), Some(16_000)),
    );
    bench.catalog.upsert(
        PostId::new("b"),
        progressive_meta(Some(2_000_000), Some(4_000)),
    );
    bench
        .present
        .insert(PostId::new("a"), vec![ByteRange::new(0, 2_000_000)]);
    bench.focus = focus_at(&["a", "b"], 0, watch_ms);
    bench
}

#[test]
fn watch_time_past_the_threshold_finishes_the_current_tail_first() {
    let requests = bench(3_000).run();

    let first = &requests[0];
    assert_eq!(first.tier, Tier::T1CurrentTail);
    assert_eq!(first.chunk.post, PostId::new("a"));
    assert_eq!(first.chunk.range.start, 2_000_000);
}

#[test]
fn committed_tail_chunks_come_in_file_order_before_startability() {
    let requests = bench(3_000).run();

    let tail_starts: Vec<u64> = requests
        .iter()
        .take_while(|request| request.tier == Tier::T1CurrentTail)
        .map(|request| request.chunk.range.start)
        .collect();
    assert_eq!(tail_starts.len(), 6);
    assert!(tail_starts.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(requests[6].tier, Tier::T2Startability);
    assert_eq!(requests[6].chunk.post, PostId::new("b"));
}

#[test]
fn watch_time_below_the_threshold_is_not_commitment() {
    let requests = bench(2_999).run();

    assert!(requests
        .iter()
        .all(|request| request.tier != Tier::T1CurrentTail));
}
