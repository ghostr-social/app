use crate::api::engine_control::FfiDataUsageLevel;
use crate::engine::DataUsageLevel;

#[test]
fn maps_the_three_data_usage_levels() {
    assert_eq!(
        DataUsageLevel::from(FfiDataUsageLevel::Conservative),
        DataUsageLevel::Conservative
    );
    assert_eq!(
        DataUsageLevel::from(FfiDataUsageLevel::Balanced),
        DataUsageLevel::Balanced
    );
    assert_eq!(
        DataUsageLevel::from(FfiDataUsageLevel::Aggressive),
        DataUsageLevel::Aggressive
    );
}
