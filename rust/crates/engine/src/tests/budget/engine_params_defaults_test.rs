use crate::{DataUsageLevel, EngineParams};

#[test]
fn defaults_match_the_plan_parameter_table() {
    let params = EngineParams::default();

    assert_eq!(params.chunk_bytes, 1024 * 1024);
    assert_eq!(params.commitment_ms, 3_000);
    assert_eq!(params.assumed_bitrate_bps, 2_500_000);
}

#[test]
fn concurrency_scales_with_the_data_usage_level() {
    let params = EngineParams::default();
    let cases = [
        (DataUsageLevel::Conservative, 2),
        (DataUsageLevel::Balanced, 3),
        (DataUsageLevel::Aggressive, 4),
    ];

    for (level, expected) in cases {
        assert_eq!(params.concurrency(level), expected, "{level:?}");
    }
}
