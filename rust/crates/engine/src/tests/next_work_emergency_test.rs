use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, PostId};

// Current post: 8 MB / 16 s, head (2 MB) already on disk, tail missing.
fn bench() -> WorkBench {
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
    bench.focus = focus_at(&["a", "b"], 0, 0);
    bench
}

#[test]
fn a_starving_player_buffer_promotes_the_current_tail_to_t0() {
    let mut bench = bench();
    bench.demand.buffer_below_emergency = true;

    let requests = bench.run();

    let first = &requests[0];
    assert_eq!(first.tier, Tier::T0PlaybackEmergency);
    assert_eq!(first.chunk.post, PostId::new("a"));
    assert_eq!(first.chunk.range.start, 2_000_000);
}

#[test]
fn gateway_demand_promotes_the_current_tail_to_t0() {
    let mut bench = bench();
    bench.demand.gateway_demand = true;

    let requests = bench.run();

    assert_eq!(requests[0].tier, Tier::T0PlaybackEmergency);
    assert_eq!(requests[0].chunk.post, PostId::new("a"));
}

#[test]
fn without_demand_the_uncommitted_current_tail_waits_in_hunger() {
    let requests = bench().run();

    assert!(requests
        .iter()
        .all(|request| request.chunk.post.as_str() == "b"));
    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T2Startability));
}
