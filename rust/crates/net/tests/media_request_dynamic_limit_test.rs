mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn growth_dispatches_waiters_and_shrink_drains_without_revocation() {
    let mut a = HeldOrigin::serve().await;
    let mut b = HeldOrigin::serve().await;
    let mut c = HeldOrigin::serve().await;
    let requests = executor(1);
    let active_a = tokio::spawn(open(requests.clone(), a.url.clone()));
    a.expect_hit().await;
    let active_b = tokio::spawn(open(requests.clone(), b.url.clone()));
    b.expect_quiet().await;

    requests.update_limits(MediaRequestLimits::try_new(2, 1).unwrap());
    b.expect_hit().await;
    requests.update_limits(MediaRequestLimits::try_new(1, 1).unwrap());
    let waiting_c = tokio::spawn(open(requests, c.url.clone()));
    c.expect_quiet().await;

    a.release_one();
    drop(active_a.await.unwrap());
    c.expect_quiet().await;
    b.release_one();
    drop(active_b.await.unwrap());
    c.expect_hit().await;
    c.release_one();
    drop(waiting_c.await.unwrap());
}

fn executor(global: usize) -> MediaRequestExecutor {
    MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(global, 1).unwrap(),
    )
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
