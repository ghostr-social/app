use crate::budget::params_for;
use crate::{DataUsageLevel, EngineParams};

const LEVELS: [(DataUsageLevel, usize); 3] = [
    (DataUsageLevel::Conservative, 2),
    (DataUsageLevel::Balanced, 3),
    (DataUsageLevel::Aggressive, 4),
];

#[test]
fn data_usage_changes_only_the_connection_budget() {
    let base = EngineParams::default();

    for (level, concurrency) in LEVELS {
        let expected = EngineParams {
            conservative_concurrency: concurrency,
            balanced_concurrency: concurrency,
            aggressive_concurrency: concurrency,
            ..base
        };
        assert_eq!(params_for(level, base), expected, "{level:?}");
    }
}

#[test]
fn each_level_pins_its_concurrency_for_every_lookup() {
    for (level, expected) in LEVELS {
        let params = params_for(level, EngineParams::default());
        for lookup in LEVELS.map(|(value, _)| value) {
            assert_eq!(params.concurrency(lookup), expected, "{level:?}");
        }
    }
}
