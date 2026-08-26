use crate::manager::pressure::StorePressure;
use core::time::Duration;

#[test]
fn one_store_refusal_decision_is_reported_once() {
    let mut pressure = StorePressure::new(Duration::ZERO);

    assert_eq!(pressure.report(0, 4), None);
    assert_eq!(pressure.report(1, 4), Some(4));
    assert_eq!(pressure.report(1, 8), None);
    assert_eq!(pressure.report(2, 8), Some(8));
}
