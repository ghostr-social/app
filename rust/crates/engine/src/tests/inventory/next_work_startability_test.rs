use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, PostId};

const MIB: u64 = 1024 * 1024;

// size == head budget: the whole 2 MB file is startability work.
fn bench() -> WorkBench {
    let mut bench = WorkBench::new();
    for post in ["a", "b"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(2_000_000), Some(4_000)),
        );
    }
    bench.focus = focus_at(&["a", "b"], 0, 0);
    bench
}

#[test]
fn hunger_starts_current_then_lets_ahead_catch_up() {
    let requests = bench().run();

    let chunks: Vec<(&str, u64)> = requests
        .iter()
        .map(|request| (request.chunk.post.as_str(), request.chunk.range.start))
        .collect();
    assert_eq!(chunks[0], ("a", 0));
    assert_eq!(chunks[1], ("b", 0));
    let current_depth = chunks.iter().position(|chunk| *chunk == ("a", MIB));
    let ahead_seed = chunks.iter().position(|chunk| *chunk == ("b", 0));
    assert!(ahead_seed < current_depth);
}

#[test]
fn all_head_work_below_target_is_startability_tier() {
    let requests = bench().run();

    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T2Startability));
}

#[test]
fn already_fetched_head_chunks_are_not_requested_again() {
    let mut bench = bench();
    bench
        .present
        .insert(PostId::new("a"), vec![ByteRange::new(0, MIB)]);

    let requests = bench.run();

    assert_eq!(requests[0].chunk.post, PostId::new("b"));
    assert!(requests.iter().any(|request| {
        request.chunk.post == PostId::new("a")
            && request.chunk.range == ByteRange::new(MIB, 2_000_000)
    }));
    assert!(requests.iter().all(|request| {
        request.chunk.post != PostId::new("a") || request.chunk.range.start >= MIB
    }));
}
