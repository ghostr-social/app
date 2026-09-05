mod request_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::internet_allowance::{InternetAllowance, InternetDataLimit};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn overrun_closes_the_body_and_cannot_be_polled_again() {
    let mut origin = HeldOrigin::serve().await;
    let ledger = InternetAllowance::memory(InternetDataLimit::Bytes(0));
    let requests = MediaRequestExecutor::with_allowance(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("fixture"),
        ledger.clone(),
    );
    let admitted = requests
        .get(&origin.url, PreemptionAuthority::PlaybackCritical)
        .expect("fixture")
        .body_limit(0)
        .admit()
        .await
        .expect("fixture");
    let mut response = admitted
        .send_with_redirect_deadline(tokio::time::Instant::now() + Duration::from_secs(1))
        .await
        .expect("fixture");
    origin.expect_hit().await;
    origin.release_one();

    assert!(response.chunk().await.is_err());
    assert!(
        response.chunk().await.is_err(),
        "failed bodies must remain closed"
    );
    assert_eq!(ledger.usage().0, 1, "observed overrun must be visible");
    assert_eq!(ledger.usage().1, 0);
}
