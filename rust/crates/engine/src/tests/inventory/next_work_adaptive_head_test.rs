use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn each_host_gets_its_own_bounded_startup_prefix() {
    let mut bench = WorkBench::new();
    for post in ["fast", "risky"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(10_000_000), Some(10_000)),
        );
    }
    bench.focus = focus_at(&["fast", "risky"], 0, 0);
    bench.head_seconds.insert(PostId::new("fast"), 2);
    bench.head_seconds.insert(PostId::new("risky"), 6);

    let requests = bench.run();
    let fast_end = last_end(&requests, "fast");
    let risky_end = last_end(&requests, "risky");

    assert_eq!(fast_end, 2_000_000);
    assert_eq!(risky_end, bench.params.head_cap_bytes);
    assert!(fast_end < risky_end);
}

fn last_end(requests: &[crate::scoring::ChunkRequest], post: &str) -> u64 {
    requests
        .iter()
        .filter(|request| request.chunk.post.as_str() == post)
        .map(|request| request.chunk.range.end)
        .max()
        .expect("planned startup range")
}
