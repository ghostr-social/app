use crate::engine::tests::scheduling_support::{focus_at, WorkBench};
use crate::engine::tests::support::progressive_meta;
use crate::engine::tiers::Tier;
use crate::engine::{ByteRange, PostId};

// 8 MB file, unknown duration: assumed bitrate gives a 1.25 MB head,
// and moov may sit at the end so the last 256 KiB must be probed.
fn bench() -> WorkBench {
    let mut bench = WorkBench::new();
    bench
        .catalog
        .upsert(PostId::new("p"), progressive_meta(Some(8_000_000), None));
    bench.focus = focus_at(&["p"], 0, 0);
    bench
}

#[test]
fn unknown_duration_adds_the_tail_probe_to_startability_work() {
    let requests = bench().run();

    let ranges: Vec<ByteRange> = requests.iter().map(|request| request.chunk.range).collect();
    assert_eq!(
        ranges,
        vec![
            ByteRange::new(0, 1_048_576),
            ByteRange::new(1_048_576, 1_250_000),
            ByteRange::new(7_737_856, 8_000_000),
        ]
    );
    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T2Startability));
}

#[test]
fn tail_chunks_wait_until_head_and_probe_are_on_disk() {
    let requests = bench().run();

    assert!(requests
        .iter()
        .all(|request| request.chunk.range.start < 1_250_000
            || request.chunk.range.start == 7_737_856));
}

// With head and probe on disk the lone-post window satisfies its
// target, so the controller lands in comfort and the tail deepens.
#[test]
fn once_startable_the_remaining_tail_deepens_in_comfort() {
    let mut bench = bench();
    bench.present.insert(
        PostId::new("p"),
        vec![
            ByteRange::new(0, 1_250_000),
            ByteRange::new(7_737_856, 8_000_000),
        ],
    );

    let requests = bench.run();

    assert_eq!(requests.len(), 7);
    assert_eq!(
        requests[0].chunk.range,
        ByteRange::new(1_250_000, 2_298_576)
    );
    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T3Deepening));
}
