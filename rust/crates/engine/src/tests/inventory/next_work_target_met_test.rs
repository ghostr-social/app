use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, EngineParams, PostId};

// Whole 2 MB files are their own heads. The startable window is
// narrowed to 1, so the fully cached current post "a" satisfies the
// target on its own and "c" (distance 1) sits beyond the window.
fn bench() -> WorkBench {
    let mut bench = WorkBench::new();
    for post in ["a", "c"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(2_000_000), Some(4_000)),
        );
    }
    bench
        .present
        .insert(PostId::new("a"), vec![ByteRange::new(0, 2_000_000)]);
    bench.params.startable_window = 1;
    bench.params.startable_target = 1;
    bench.focus = focus_at(&["a", "c"], 0, 0);
    bench
}

#[test]
fn heads_beyond_the_satisfied_window_are_speculative() {
    let requests = bench().run();

    assert!(!requests.is_empty());
    assert!(requests
        .iter()
        .all(|request| request.chunk.post.as_str() == "c"));
    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T4Speculative));
}

#[test]
fn a_fully_satisfied_window_requests_nothing() {
    let mut bench = bench();
    bench
        .present
        .insert(PostId::new("c"), vec![ByteRange::new(0, 2_000_000)]);

    assert!(bench.run().is_empty());
}

#[test]
fn an_unmet_target_keeps_upcoming_heads_at_startability() {
    let mut bench = bench();
    bench.params = EngineParams::default();
    bench.present.clear();

    let requests = bench.run();

    let c_tier = requests
        .iter()
        .find(|request| request.chunk.post.as_str() == "c")
        .map(|request| request.tier);
    assert_eq!(c_tier, Some(Tier::T2Startability));
}
