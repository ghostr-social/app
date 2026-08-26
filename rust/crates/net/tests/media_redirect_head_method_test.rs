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
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );

    let response = executor
        .get(&start, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .head()
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert!(target.hit().await.starts_with("HEAD /redirected "));
}
