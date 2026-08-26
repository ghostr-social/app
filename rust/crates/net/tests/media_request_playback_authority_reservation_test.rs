mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::request::open;
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn same_authority_keeps_one_socket_slot_for_playback() {
    let mut origin = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(3, 2).expect("valid test fixture"),
    );
    let first = open(
        requests.clone(),
        origin.url.clone(),
        PreemptionAuthority::Speculative,
    )
    .await;
    origin.expect_hit().await;

    let waiting = tokio::spawn(open(
        requests.clone(),
        origin.url.clone(),
        PreemptionAuthority::Transition,
    ));
    origin.expect_quiet().await;
    let critical = tokio::spawn(open(
        requests,
        origin.url.clone(),
        PreemptionAuthority::PlaybackCritical,
    ));
    origin.expect_hit().await;
    let critical = critical.await.expect("valid test fixture");
    origin.expect_quiet().await;

    origin.release_one();
    drop(critical);
    origin.expect_quiet().await;
    origin.release_one();
    drop(first);
    origin.expect_hit().await;
    origin.release_one();
    drop(waiting.await.expect("valid test fixture"));
}
