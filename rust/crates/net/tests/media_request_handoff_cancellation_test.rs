mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn cancellation_after_handoff_returns_capacity() {
    let mut held = HeldOrigin::serve().await;
    let mut cancelled = HeldOrigin::serve().await;
    let mut next = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let active = open(requests.clone(), held.url.clone()).await;
    held.expect_hit().await;
    let handed_off = tokio::spawn(admit(requests.clone(), cancelled.url.clone()));
    cancelled.expect_quiet().await;

    held.release_one();
    drop(active);
    let admitted = handed_off.await.unwrap();
    drop(admitted);
    let successor = tokio::spawn(open(requests, next.url.clone()));

    next.expect_hit().await;
    next.release_one();
    drop(successor.await.unwrap());
}

async fn admit(
    requests: MediaRequestExecutor,
    url: String,
) -> ghostr_net::media_request_executor::AdmittedMediaRequest {
    requests
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
}

async fn open(
    requests: MediaRequestExecutor,
    url: String,
) -> ghostr_net::media_request_executor::MediaResponse {
    requests
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await
        .unwrap()
}
