use crate::engine::chunk_plan::ChunkPlan;
use crate::engine::tests::support::progressive_meta;
use crate::engine::{ByteRange, EngineParams};

const MIB: u64 = 1024 * 1024;

fn plan(size: Option<u64>, bitrate_bps: u64) -> ChunkPlan {
    let meta = progressive_meta(size, Some(10_000));
    ChunkPlan::for_meta(&meta, bitrate_bps, &EngineParams::default())
}

#[test]
fn the_head_splits_into_one_megabyte_chunks_with_a_short_last_chunk() {
    // Head budget: 1_250_000 bytes at the default assumed bitrate.
    let ranges = plan(None, 2_500_000).head_ranges();

    let expected = vec![
        ByteRange::new(0, MIB),
        ByteRange::new(MIB, 1_250_000),
    ];
    assert_eq!(ranges, expected);
}

#[test]
fn tail_chunks_cover_from_the_head_to_the_end_of_file() {
    let ranges = plan(Some(3_500_000), 2_500_000).tail_ranges();

    let expected = vec![
        ByteRange::new(1_250_000, 1_250_000 + MIB),
        ByteRange::new(1_250_000 + MIB, 1_250_000 + 2 * MIB),
        ByteRange::new(1_250_000 + 2 * MIB, 3_500_000),
    ];
    assert_eq!(ranges, expected);
}

#[test]
fn an_unknown_size_plans_no_tail() {
    assert!(plan(None, 2_500_000).tail_ranges().is_empty());
}

#[test]
fn a_file_swallowed_by_the_head_plans_no_tail() {
    assert!(plan(Some(500_000), 2_500_000).tail_ranges().is_empty());
}
