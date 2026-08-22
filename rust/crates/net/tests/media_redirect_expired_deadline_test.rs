mod redirect_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    MediaRequestAdmissionTimeout, MediaRequestExecutor, MediaRequestLimits,
};
use redirect_gate_fixture::target::TargetOrigin;
use redirect_gate_fixture::{delayed_redirect_origin, OneHopClient};
use std::time::Duration;

#[tokio::test]
async fn an_expired_redirect_deadline_cannot_admit_an_immediately_free_target() {
    let mut target = TargetOrigin::serve().await;
    let (start, contacted) =
        delayed_redirect_origin(target.redirected_url.clone(), Duration::from_millis(60)).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let admitted = executor
        .get(&start, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap();

    let result = admitted
        .send_with_redirect_deadline(tokio::time::Instant::now() + Duration::from_millis(30))
        .await;
    let Err(error) = result else {
        panic!("expired redirect deadline was admitted");
    };
    assert!(error.is::<MediaRequestAdmissionTimeout>());
    contacted.await.expect("initial origin was contacted");
    target.quiet().await;
}
