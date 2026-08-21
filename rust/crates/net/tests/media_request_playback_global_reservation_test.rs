mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::request::open;
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn independent_authorities_keep_one_global_socket_slot_for_playback() {
    let mut first_origin = HeldOrigin::serve().await;
    let mut waiting_origin = HeldOrigin::serve().await;
    let mut critical_origin = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(2, 1).unwrap(),
    );
    let first = open(
        requests.clone(),
        first_origin.url.clone(),
        PreemptionAuthority::Transition,
    )
    .await;
    first_origin.expect_hit().await;

    let waiting = tokio::spawn(open(
        requests.clone(),
        waiting_origin.url.clone(),
        PreemptionAuthority::Speculative,
    ));
    waiting_origin.expect_quiet().await;
    let critical = tokio::spawn(open(
        requests,
        critical_origin.url.clone(),
        PreemptionAuthority::PlaybackCritical,
    ));
    critical_origin.expect_hit().await;
    let critical = critical.await.unwrap();
    waiting_origin.expect_quiet().await;

    critical_origin.release_one();
    drop(critical);
    waiting_origin.expect_quiet().await;
    first_origin.release_one();
    drop(first);
    waiting_origin.expect_hit().await;
    waiting_origin.release_one();
    drop(waiting.await.unwrap());
}
