mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits, MediaResponse};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn saturated_authority_cannot_hoard_independent_global_capacity() {
    let mut a = HeldOrigin::serve().await;
    let mut b = HeldOrigin::serve().await;
    let mut c = HeldOrigin::serve().await;
    let requests = executor(2, 1);

    let first_a = open(requests.clone(), a.url.clone()).await;
    a.expect_hit().await;
    let second_a = tokio::spawn(open(requests.clone(), a.url.clone()));
    a.expect_quiet().await;

    let first_b = open(requests.clone(), b.url.clone()).await;
    b.expect_hit().await;
    let first_c = tokio::spawn(open(requests.clone(), c.url.clone()));
    c.expect_quiet().await;

    a.release_one();
    drop(first_a);
    a.expect_hit().await;
    let second_a = second_a.await.unwrap();
    c.expect_quiet().await;

    b.release_one();
    drop(first_b);
    c.expect_hit().await;
    let first_c = first_c.await.unwrap();
    a.release_one();
    c.release_one();
    drop((second_a, first_c));
}

fn executor(global: usize, authority: usize) -> MediaRequestExecutor {
    let limits = MediaRequestLimits::try_new(global, authority).unwrap();
    MediaRequestExecutor::new(LocalMediaClient::shared(), limits)
}

async fn open(requests: MediaRequestExecutor, url: String) -> MediaResponse {
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
