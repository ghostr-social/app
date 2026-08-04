use crate::engine::budget::params_for;
use crate::engine::{DataUsageLevel, EngineParams};

const MIB: u64 = 1024 * 1024;

const LEVELS: [DataUsageLevel; 3] = [
    DataUsageLevel::Conservative,
    DataUsageLevel::Balanced,
    DataUsageLevel::Aggressive,
];

#[test]
fn conservative_halves_the_head_budget_and_narrows_the_lookahead() {
    let params = params_for(DataUsageLevel::Conservative, EngineParams::default());

    assert_eq!(params.head_seconds, 2);
    assert_eq!(params.head_cap_bytes, 3 * MIB / 2);
    assert_eq!(params.startable_target, 3);
    assert_eq!(params.startable_window, 4);
    assert_eq!(params.chunk_bytes, MIB);
}

#[test]
fn balanced_keeps_the_base_head_budget_and_lookahead() {
    let params = params_for(DataUsageLevel::Balanced, EngineParams::default());

    assert_eq!(params.head_seconds, 4);
    assert_eq!(params.head_cap_bytes, 3 * MIB);
    assert_eq!(params.startable_target, 4);
    assert_eq!(params.startable_window, 6);
    assert_eq!(params.commitment_ms, 3_000);
}

#[test]
fn aggressive_grows_the_head_budget_and_widens_the_lookahead() {
    let params = params_for(DataUsageLevel::Aggressive, EngineParams::default());

    assert_eq!(params.head_seconds, 6);
    assert_eq!(params.head_cap_bytes, 6 * MIB);
    assert_eq!(params.startable_target, 5);
    assert_eq!(params.startable_window, 8);
    assert_eq!(params.emergency_buffer_s, 5);
}

#[test]
fn each_level_pins_its_concurrency_for_every_lookup() {
    let expectations = [2usize, 3, 4];

    for (level, expected) in LEVELS.into_iter().zip(expectations) {
        let params = params_for(level, EngineParams::default());
        for lookup in LEVELS {
            assert_eq!(params.concurrency(lookup), expected, "{level:?}");
        }
    }
}

#[test]
fn conservative_never_scales_the_target_or_window_to_zero() {
    let tiny = EngineParams {
        startable_target: 1,
        startable_window: 2,
        ..EngineParams::default()
    };

    let params = params_for(DataUsageLevel::Conservative, tiny);

    assert_eq!(params.startable_target, 1);
    assert_eq!(params.startable_window, 1);
}
