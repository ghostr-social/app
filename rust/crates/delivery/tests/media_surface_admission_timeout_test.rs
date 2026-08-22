#[path = "media_surface_admission_timeout_test/fixture.rs"]
mod fixture;
mod range_fixture;

use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;

#[tokio::test]
async fn body_and_head_enforce_their_local_admission_deadline() {
    let observed = fixture::exercise().await;

    assert!(observed.progressive.is::<MediaRequestAdmissionTimeout>());
    assert!(observed.head.is::<MediaRequestAdmissionTimeout>());
    assert_eq!(observed.failure_ratio, 0.0);
}
