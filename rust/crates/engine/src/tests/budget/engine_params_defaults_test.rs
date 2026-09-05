use crate::{DataUsageLevel, EngineParams};

#[test]
fn defaults_match_the_plan_parameter_table() {
    let params = EngineParams::default();

    assert_eq!(params.chunk_bytes, 1024 * 1024);
    assert_eq!(params.commitment_ms, 3_000);
    assert_eq!(params.assumed_bitrate_bps, 2_500_000);
}

#[test]
fn default_concurrency_requires_measurement_before_expansion() {
    let params = EngineParams::default();
    let cases = [
        (DataUsageLevel::Conservative, 2),
        (DataUsageLevel::Balanced, 2),
        (DataUsageLevel::Aggressive, 2),
    ];

    for (level, expected) in cases {
        assert_eq!(params.concurrency(level), expected, "{level:?}");
    }
}
