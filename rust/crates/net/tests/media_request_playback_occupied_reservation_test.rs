mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::request::open;
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn playback_occupying_the_reservation_leaves_ordinary_capacity_usable() {
    let mut origin = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(2, 2).unwrap(),
    );
    let critical = tokio::spawn(open(
        requests.clone(),
        origin.url.clone(),
        PreemptionAuthority::PlaybackCritical,
    ));
    origin.expect_hit().await;
    let speculative = tokio::spawn(open(
        requests,
        origin.url.clone(),
        PreemptionAuthority::Speculative,
    ));
    origin.expect_hit().await;

    origin.release_one();
    origin.release_one();
    drop((critical.await.unwrap(), speculative.await.unwrap()));
}
