use rust_lib_ghostr::video::outbound_media_client::MediaHttpTimeouts;
use std::time::Duration;

#[test]
fn requires_positive_connect_and_request_timeouts() {
    assert!(MediaHttpTimeouts::new(Duration::ZERO, Duration::from_secs(1)).is_err());
    assert!(MediaHttpTimeouts::new(Duration::from_secs(1), Duration::ZERO).is_err());
    assert!(MediaHttpTimeouts::new(Duration::from_secs(1), Duration::from_secs(2)).is_ok());
}
