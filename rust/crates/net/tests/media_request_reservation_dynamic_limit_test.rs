mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::request::open;
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn resized_gate_reserves_after_growth_and_drains_after_shrink() {
    let mut active_origin = HeldOrigin::serve().await;
    let mut waiting_origin = HeldOrigin::serve().await;
    let mut critical_origin = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let active = open(
        requests.clone(),
        active_origin.url.clone(),
        PreemptionAuthority::Speculative,
    )
    .await;
    active_origin.expect_hit().await;
    let waiting = tokio::spawn(open(
        requests.clone(),
        waiting_origin.url.clone(),
        PreemptionAuthority::Transition,
    ));
    let critical = tokio::spawn(open(
        requests.clone(),
        critical_origin.url.clone(),
        PreemptionAuthority::PlaybackCritical,
    ));
    waiting_origin.expect_quiet().await;
    critical_origin.expect_quiet().await;

    requests.update_limits(MediaRequestLimits::try_new(2, 2).expect("valid test fixture"));
    critical_origin.expect_hit().await;
    let critical = critical.await.expect("valid test fixture");
    waiting_origin.expect_quiet().await;
    critical_origin.release_one();
    drop(critical);
    waiting_origin.expect_quiet().await;

    requests.update_limits(MediaRequestLimits::try_new(1, 1).expect("valid test fixture"));
    let active_authority =
        RequestAuthority::from_url(&active_origin.url).expect("valid test fixture");
    assert_eq!(requests.active_for(&active_authority), 1);
    active_origin.release_one();
    drop(active);
    waiting_origin.expect_hit().await;
    waiting_origin.release_one();
    drop(waiting.await.expect("valid test fixture"));
}
