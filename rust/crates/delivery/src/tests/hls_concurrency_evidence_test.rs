use crate::manager::concurrency::{observed_admitted_capacity, observed_claimed_requests};

#[test]
fn hls_and_progressive_requests_share_one_capacity_observation() {
    assert_eq!(observed_claimed_requests(1, 2), 3);
    assert_eq!(observed_admitted_capacity(1, 3, 3), 3);
    assert_eq!(observed_admitted_capacity(1, 4, 3), 3);
}
