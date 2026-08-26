mod redirect_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    MediaRequestAdmissionTimeout, MediaRequestExecutor, MediaRequestLimits,
};
use redirect_gate_fixture::target::TargetOrigin;
use redirect_gate_fixture::{redirect_origin, OneHopClient};

#[tokio::test]
async fn redirect_gate_expiry_is_a_typed_local_admission_timeout() {
    let mut target = TargetOrigin::serve().await;
    let start = redirect_origin(target.redirected_url.clone()).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(2, 1).expect("valid test fixture"),
    );
    let held = executor
        .get(&target.held_url, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture");
    let _ = target.hit().await;
    let admitted = executor
        .get(&start, PreemptionAuthority::PlaybackCritical)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture");

    let deadline = tokio::time::Instant::now() + Duration::from_millis(40);
    let result = admitted.send_with_redirect_deadline(deadline).await;
    let Err(error) = result else {
        panic!("redirect authority wait escaped its deadline");
    };
    assert!(error.is::<MediaRequestAdmissionTimeout>());
    target.quiet().await;
    drop(held);
}
