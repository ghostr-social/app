mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::request::open;
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn single_socket_capacity_still_admits_lower_priority_work() {
    let mut origin = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let request = tokio::spawn(open(
        requests,
        origin.url.clone(),
        PreemptionAuthority::Speculative,
    ));

    origin.expect_hit().await;
    origin.release_one();
    drop(request.await.expect("valid test fixture"));
}
