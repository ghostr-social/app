use crate::api::focus_mapping::parse_data_usage;
use crate::engine::DataUsageLevel;

#[test]
fn maps_the_three_data_usage_levels() {
    assert_eq!(
        parse_data_usage("conservative").expect("conservative"),
        DataUsageLevel::Conservative
    );
    assert_eq!(
        parse_data_usage("balanced").expect("balanced"),
        DataUsageLevel::Balanced
    );
    assert_eq!(
        parse_data_usage("aggressive").expect("aggressive"),
        DataUsageLevel::Aggressive
    );
}

#[test]
fn rejects_an_unknown_data_usage_level() {
    let error = parse_data_usage("turbo").expect_err("unknown level");

    assert!(error.to_string().contains("turbo"));
}
