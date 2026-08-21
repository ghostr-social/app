mod redirect_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use redirect_gate_fixture::target::TargetOrigin;
use redirect_gate_fixture::{redirect_origin, OneHopClient};

#[tokio::test]
async fn head_remains_head_at_every_redirect_hop() {
    let mut target = TargetOrigin::serve().await;
    let start = redirect_origin(target.redirected_url.clone()).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );

    let response = executor
        .get(&start, PreemptionAuthority::Transition)
        .unwrap()
        .head()
        .admit()
        .await
        .unwrap()
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert!(target.hit().await.starts_with("HEAD /redirected "));
}
