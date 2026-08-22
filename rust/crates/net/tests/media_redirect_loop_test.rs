mod redirect_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use redirect_gate_fixture::{loop_origin, OneHopClient};

#[tokio::test]
async fn a_redirect_loop_fails_without_recontacting_the_origin() {
    let url = loop_origin().await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let result = executor
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await;

    let Err(error) = result else {
        panic!("redirect loop was accepted");
    };
    assert!(format!("{error:#}").contains("media redirect loop detected"));
}
