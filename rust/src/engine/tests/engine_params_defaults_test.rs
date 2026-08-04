use crate::engine::{DataUsageLevel, EngineParams};

#[test]
fn defaults_match_the_plan_parameter_table() {
    let params = EngineParams::default();

    assert_eq!(params.head_seconds, 4);
    assert_eq!(params.head_cap_bytes, 3 * 1024 * 1024);
    assert_eq!(params.chunk_bytes, 1024 * 1024);
    assert_eq!(params.startable_target, 4);
    assert_eq!(params.startable_window, 6);
    assert_eq!(params.commitment_ms, 3_000);
    assert_eq!(params.emergency_buffer_s, 5);
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
