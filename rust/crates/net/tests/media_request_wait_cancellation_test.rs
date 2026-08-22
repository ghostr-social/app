mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn cancelling_a_queued_request_cannot_leak_capacity() {
    let mut held = HeldOrigin::serve().await;
    let mut cancelled = HeldOrigin::serve().await;
    let mut next = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let active = tokio::spawn(open(requests.clone(), held.url.clone()));
    held.expect_hit().await;
    let waiting = tokio::spawn(open(requests.clone(), cancelled.url.clone()));
    cancelled.expect_quiet().await;
    waiting.abort();
    assert!(waiting.await.is_err(), "queued request is cancelled");
    let successor = tokio::spawn(open(requests, next.url.clone()));

    held.release_one();
    drop(active.await.unwrap());
    next.expect_hit().await;
    cancelled.expect_quiet().await;
    next.release_one();
    drop(successor.await.unwrap());
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
