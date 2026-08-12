use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, PostId};

const HEAD_END: u64 = 2_000_000;

#[test]
fn persisted_allowance_stops_ahead_tail_without_blocking_current_demand() {
    let mut bench = bench();
    let allowance_end = HEAD_END + bench.params.chunk_bytes;

    let ahead: Vec<_> = for_post(&bench.run(), "ahead");
    assert_eq!(ahead, vec![ByteRange::new(HEAD_END, allowance_end)]);

    bench
        .present
        .insert(PostId::new("ahead"), vec![ByteRange::new(0, allowance_end)]);
    assert!(for_post(&bench.run(), "ahead").is_empty());

    bench.present.insert(
        PostId::new("current"),
        vec![ByteRange::new(0, allowance_end)],
    );
    bench.tail_end.insert(
        PostId::new("current"),
        allowance_end + bench.params.chunk_bytes,
    );
    bench.demand.gateway_demand = true;
    let requests = bench.run();
    let current = requests
        .iter()
        .find(|request| request.chunk.post.as_str() == "current")
        .expect("gateway demand advances beyond speculative allowance");
    assert_eq!(current.tier, Tier::T0PlaybackEmergency);
    assert_eq!(current.chunk.range.start, allowance_end);
}

fn bench() -> WorkBench {
    let mut bench = WorkBench::new();
    for post in ["current", "ahead"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(8_000_000), Some(16_000)),
        );
        bench
            .present
            .insert(PostId::new(post), vec![ByteRange::new(0, HEAD_END)]);
    }
    bench.focus = focus_at(&["current", "ahead"], 0, 0);
    bench
}

fn for_post(requests: &[crate::scoring::ChunkRequest], post: &str) -> Vec<ByteRange> {
    requests
        .iter()
        .filter(|request| request.chunk.post.as_str() == post)
        .map(|request| request.chunk.range)
        .collect()
}
