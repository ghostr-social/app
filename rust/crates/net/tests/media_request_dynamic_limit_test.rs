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

    requests.update_limits(MediaRequestLimits::try_new(3, 1).expect("valid test fixture"));
    b.expect_hit().await;
    requests.update_limits(MediaRequestLimits::try_new(1, 1).expect("valid test fixture"));
    let waiting_c = tokio::spawn(open(requests, c.url.clone()));
    c.expect_quiet().await;

    a.release_one();
    drop(active_a.await.expect("valid test fixture"));
    c.expect_quiet().await;
    b.release_one();
    drop(active_b.await.expect("valid test fixture"));
    c.expect_hit().await;
    c.release_one();
    drop(waiting_c.await.expect("valid test fixture"));
}

fn executor(global: usize) -> MediaRequestExecutor {
    MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(global, 1).expect("valid test fixture"),
    )
}

async fn open(
    requests: MediaRequestExecutor,
    url: String,
) -> ghostr_net::media_request_executor::MediaResponse {
    requests
        .get(&url, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture")
}
