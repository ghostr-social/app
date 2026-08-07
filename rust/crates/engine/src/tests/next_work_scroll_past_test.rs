use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::PostId;

fn bench() -> WorkBench {
    let mut bench = WorkBench::new();
    for post in ["a", "b"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(2_000_000), Some(4_000)),
        );
    }
    bench
}

#[test]
fn a_post_leaving_the_window_yields_no_work() {
    let mut bench = bench();
    bench.focus = focus_at(&["a", "b"], 0, 0);
    let before = bench.run();
    assert!(before
        .iter()
        .any(|request| request.chunk.post.as_str() == "a"));

    bench.focus = focus_at(&["b"], 0, 0);
    let after = bench.run();

    assert!(after
        .iter()
        .all(|request| request.chunk.post.as_str() == "b"));
    assert!(!after.is_empty());
}

#[test]
fn an_uncatalogued_post_in_the_window_yields_no_work() {
    let mut bench = bench();
    bench.focus = focus_at(&["b", "mystery"], 0, 0);

    let requests = bench.run();

    assert!(requests
        .iter()
        .all(|request| request.chunk.post.as_str() == "b"));
}

#[test]
fn an_empty_window_yields_no_work() {
    let bench = bench();

    assert!(bench.run().is_empty());
}
