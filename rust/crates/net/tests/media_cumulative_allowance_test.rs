mod request_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::internet_allowance::{InternetAllowance, InternetDataLimit};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn exhausted_cumulative_allowance_blocks_io_after_request_capacity_refills() {
    let mut origin = HeldOrigin::serve().await;
    let ledger = InternetAllowance::memory(InternetDataLimit::Bytes(1));
    let requests = MediaRequestExecutor::with_allowance(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("fixture"),
        ledger.clone(),
    );
    let request = requests
        .get(&origin.url, PreemptionAuthority::PlaybackCritical)
        .expect("fixture")
        .body_limit(1)
        .admit()
        .await
        .expect("fixture");
    let mut response = request
        .send_with_redirect_deadline(tokio::time::Instant::now() + Duration::from_secs(1))
        .await
        .expect("fixture");
    origin.expect_hit().await;
    origin.release_one();
    assert_eq!(
        response
            .chunk()
            .await
            .expect("fixture")
            .expect("fixture")
            .len(),
        1
    );
    assert!(response.chunk().await.expect("fixture").is_none());
    drop(response);
    requests.update_limits(MediaRequestLimits::try_new(2, 2).expect("fixture"));

    let next = requests
        .get(&origin.url, PreemptionAuthority::PlaybackCritical)
        .expect("fixture")
        .body_limit(1)
        .admit()
        .await;
    assert!(next.is_err());
    assert!(next
        .err()
        .expect("fixture")
        .is::<ghostr_net::internet_allowance::InternetAdmissionDenied>());
    origin.expect_quiet().await;
    assert_eq!(ledger.usage().0, 1);
    assert_eq!(ledger.usage().1, 0);
}
