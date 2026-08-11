use crate::concurrency::AdaptiveConcurrency;

#[test]
fn a_cap_drop_clamps_immediately_but_a_cap_raise_needs_new_evidence() {
    let mut policy = AdaptiveConcurrency::new(2, 4);

    policy.set_maximum(1);
    assert_eq!(policy.limit(), 1);
    assert_eq!(policy.accepted_limit(), 1);

    policy.set_maximum(4);
    assert_eq!(policy.limit(), 1);
    assert_eq!(policy.accepted_limit(), 1);
}
