use crate::chunk_plan::ChunkPlan;
use crate::tests::support::progressive_meta;
use crate::EngineParams;

fn head_for(size: Option<u64>, bitrate_bps: u64) -> u64 {
    let meta = progressive_meta(size, Some(10_000));
    ChunkPlan::for_meta(&meta, bitrate_bps, &EngineParams::default()).head_bytes()
}

#[test]
fn head_is_four_seconds_of_video_at_the_estimated_bitrate() {
    // 4 s * 2_500_000 bps / 8 = 1_250_000 bytes.
    assert_eq!(head_for(None, 2_500_000), 1_250_000);
}

#[test]
fn head_is_capped_at_three_megabytes_for_high_bitrates() {
    assert_eq!(head_for(None, 10_000_000), 3 * 1024 * 1024);
}

#[test]
fn the_whole_file_is_the_head_when_smaller_than_the_budget() {
    assert_eq!(head_for(Some(500_000), 2_500_000), 500_000);
}

#[test]
fn a_zero_bitrate_plans_an_empty_head() {
    assert_eq!(head_for(None, 0), 0);
}
