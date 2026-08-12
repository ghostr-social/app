use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, PostId};

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
fn watch_time_past_the_threshold_prioritizes_the_current_reserve() {
    let requests = bench(3_000).run();

    let first = &requests[0];
    assert_eq!(first.tier, Tier::T1CurrentTail);
    assert_eq!(first.chunk.post, PostId::new("a"));
    assert_eq!(first.chunk.range.start, 2_000_000);
}

#[test]
fn committed_current_gets_one_reserve_chunk_before_ahead_startability() {
    let requests = bench(3_000).run();

    let tail_starts: Vec<u64> = requests
        .iter()
        .take_while(|request| request.tier == Tier::T1CurrentTail)
        .map(|request| request.chunk.range.start)
        .collect();
    assert_eq!(tail_starts, vec![2_000_000]);
    assert_eq!(requests[1].tier, Tier::T2Startability);
    assert_eq!(requests[1].chunk.post, PostId::new("b"));
}

#[test]
fn gateway_demand_advances_current_past_its_startup_reserve() {
    let mut bench = bench(3_000);
    bench
        .present
        .insert(PostId::new("a"), vec![ByteRange::new(0, 3_000_000)]);
    bench.demand.gateway_demand = true;

    let current = bench
        .run()
        .into_iter()
        .find(|request| request.chunk.post == PostId::new("a"))
        .expect("decoder demand advances the current item");
    assert_eq!(current.tier, Tier::T0PlaybackEmergency);
    assert_eq!(current.chunk.range.start, 3_000_000);
}

#[test]
fn watch_time_below_the_threshold_is_not_commitment() {
    let requests = bench(2_999).run();

    assert!(requests
        .iter()
        .all(|request| request.tier != Tier::T1CurrentTail));
}
