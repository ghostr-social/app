use crate::chunk_plan::ChunkPlan;
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams};

const MIB: u64 = 1024 * 1024;

fn plan() -> ChunkPlan {
    // Head 1_250_000; tail chunks up to 3_500_000.
    let meta = progressive_meta(Some(3_500_000), Some(10_000));
    ChunkPlan::for_meta(&meta, 2_500_000, &EngineParams::default())
}

#[test]
fn with_nothing_fetched_the_first_head_chunk_is_next() {
    assert_eq!(plan().next_missing_chunk(&[]), Some(ByteRange::new(0, MIB)));
}

#[test]
fn chunks_covered_by_split_ranges_are_skipped() {
    let have = [ByteRange::new(0, 400_000), ByteRange::new(300_000, MIB)];

    assert_eq!(
        plan().next_missing_chunk(&have),
        Some(ByteRange::new(MIB, 1_250_000))
    );
}

#[test]
fn a_partially_covered_chunk_is_still_missing() {
    let have = [ByteRange::new(0, 400_000), ByteRange::new(500_000, MIB)];

    assert_eq!(
        plan().next_missing_chunk(&have),
        Some(ByteRange::new(0, MIB))
    );
}

#[test]
fn the_tail_is_next_once_the_head_is_complete() {
    let have = [ByteRange::new(0, 1_250_000)];

    assert_eq!(
        plan().next_missing_chunk(&have),
        Some(ByteRange::new(1_250_000, 1_250_000 + MIB))
    );
}

#[test]
fn a_fully_covered_plan_needs_nothing() {
    let have = [ByteRange::new(0, 3_500_000)];

    assert_eq!(plan().next_missing_chunk(&have), None);
}
