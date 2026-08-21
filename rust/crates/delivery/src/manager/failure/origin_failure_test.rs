use super::{origin_failure_class, FailureClass};
use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;

#[test]
fn only_remote_failures_produce_origin_evidence() {
    let local = anyhow::Error::new(MediaRequestAdmissionTimeout);
    let remote = anyhow::anyhow!("connection reset before response headers");

    assert_eq!(origin_failure_class(&local), None);
    assert_eq!(origin_failure_class(&remote), Some(FailureClass::Transient));
}
