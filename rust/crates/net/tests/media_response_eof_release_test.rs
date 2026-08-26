mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits, MediaResponse};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn reaching_body_eof_releases_a_retained_response_lease() {
    let mut first_origin = HeldOrigin::serve().await;
    let mut next_origin = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let mut first = open(requests.clone(), first_origin.url.clone()).await;
    first_origin.expect_hit().await;
    let next = tokio::spawn(open(requests, next_origin.url.clone()));
    next_origin.expect_quiet().await;

    first_origin.release_one();
    assert!(first.chunk().await.expect("valid test fixture").is_some());
    assert!(first.chunk().await.expect("valid test fixture").is_none());

    next_origin.expect_hit().await;
    next_origin.release_one();
    drop((first, next.await.expect("valid test fixture")));
}

async fn open(requests: MediaRequestExecutor, url: String) -> MediaResponse {
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
