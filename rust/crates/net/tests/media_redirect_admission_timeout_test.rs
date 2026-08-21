mod redirect_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    MediaRequestAdmissionTimeout, MediaRequestExecutor, MediaRequestLimits,
};
use redirect_gate_fixture::target::TargetOrigin;
use redirect_gate_fixture::{redirect_origin, OneHopClient};
use std::time::Duration;

#[tokio::test]
async fn redirect_gate_expiry_is_a_typed_local_admission_timeout() {
    let mut target = TargetOrigin::serve().await;
    let start = redirect_origin(target.redirected_url.clone()).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(2, 1).unwrap(),
    );
    let held = executor
        .get(&target.held_url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await
        .unwrap();
    let _ = target.hit().await;
    let admitted = executor
        .get(&start, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap();

    let deadline = tokio::time::Instant::now() + Duration::from_millis(40);
    let result = admitted.send_with_redirect_deadline(deadline).await;
    let Err(error) = result else {
        panic!("redirect authority wait escaped its deadline");
    };
    assert!(error.is::<MediaRequestAdmissionTimeout>());
    target.quiet().await;
    drop(held);
}
