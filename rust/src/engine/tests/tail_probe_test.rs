use crate::engine::chunk_plan::{ChunkPlan, TAIL_PROBE_BYTES};
use crate::engine::tests::support::progressive_meta;
use crate::engine::{ByteRange, EngineParams};

fn plan(size: Option<u64>, duration_ms: Option<u64>) -> ChunkPlan {
    let meta = progressive_meta(size, duration_ms);
    ChunkPlan::for_meta(&meta, 2_500_000, &EngineParams::default())
}

#[test]
fn an_unknown_duration_requires_a_tail_probe_for_moov() {
    assert!(plan(Some(10_000_000), None).needs_tail_probe());
    assert!(!plan(Some(10_000_000), Some(9_000)).needs_tail_probe());
}

#[test]
fn the_tail_probe_reads_the_final_quarter_megabyte() {
    let range = plan(Some(10_000_000), None).tail_probe_range();

    assert_eq!(
        range,
        Some(ByteRange::new(10_000_000 - TAIL_PROBE_BYTES, 10_000_000))
    );
}

#[test]
fn the_tail_probe_clamps_to_the_start_of_small_files() {
    let range = plan(Some(100_000), None).tail_probe_range();

    assert_eq!(range, Some(ByteRange::new(0, 100_000)));
}

#[test]
fn no_probe_range_exists_without_a_known_size() {
    assert_eq!(plan(None, None).tail_probe_range(), None);
}

#[test]
fn a_known_duration_needs_no_tail_probe_range() {
    assert_eq!(plan(Some(10_000_000), Some(9_000)).tail_probe_range(), None);
}
