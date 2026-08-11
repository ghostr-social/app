use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, PostId};

#[test]
fn safe_current_refill_yields_to_ahead_startup_before_eof() {
    let mut bench = WorkBench::new();
    bench.catalog.upsert(
        PostId::new("current"),
        progressive_meta(Some(8_000_000), Some(16_000)),
    );
    bench.catalog.upsert(
        PostId::new("ahead"),
        progressive_meta(Some(2_000_000), Some(4_000)),
    );
    bench
        .present
        .insert(PostId::new("current"), vec![ByteRange::new(0, 2_000_000)]);
    bench.focus = focus_at(&["current", "ahead"], 0, 3_000);
    bench.tail_end.insert(PostId::new("current"), 3_000_000);

    let requests = bench.run();
    let current: Vec<_> = requests
        .iter()
        .filter(|work| work.chunk.post.as_str() == "current")
        .collect();
    let ahead = requests
        .iter()
        .find(|work| work.chunk.post.as_str() == "ahead")
        .expect("ahead startup work");

    assert_eq!(current.len(), 1);
    assert_eq!(current[0].tier, Tier::T1CurrentTail);
    assert!(current[0].chunk.range.end < 8_000_000);
    assert_eq!(ahead.tier, Tier::T2Startability);
}
