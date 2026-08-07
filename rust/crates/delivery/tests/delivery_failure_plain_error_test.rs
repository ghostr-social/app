use anyhow::anyhow;
use ghostr_delivery::delivery_failure::{classify, FailureClass};

#[test]
fn ordinary_errors_remain_transient_delivery_failures() {
    assert_eq!(
        classify(&anyhow!("connection reset without a permanent marker")),
        FailureClass::Transient
    );
}
