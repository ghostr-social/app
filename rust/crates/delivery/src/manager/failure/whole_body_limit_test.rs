use super::origin_failure_class;
use ghostr_engine::adaptive::WholeBodyContract;

#[test]
fn whole_body_policy_limit_is_not_an_origin_failure() {
    let error = crate::chunk::whole_body_limit::WholeBodyLimitReached::check(
        8,
        1,
        WholeBodyContract::Capped { maximum_bytes: 8 },
    )
    .unwrap_err();

    assert_eq!(origin_failure_class(&error), None);
    let limit = crate::chunk::whole_body_limit::from_error(&error).unwrap();
    assert_eq!(limit.maximum_bytes(), 8);
    assert_eq!(limit.received_bytes(), 9);
}
