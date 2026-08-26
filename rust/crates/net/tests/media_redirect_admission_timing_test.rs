mod redirect_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use redirect_gate_fixture::target::TargetOrigin;
use redirect_gate_fixture::{redirect_origin, OneHopClient};

#[tokio::test]
async fn successful_redirect_reports_only_its_local_gate_wait() {
    let mut target = TargetOrigin::serve().await;
    let start = redirect_origin(target.delayed_url.clone()).await;
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

    let started = tokio::time::Instant::now();
    let response = admitted.send_with_redirect_deadline(started + Duration::from_secs(1));
    tokio::pin!(response);
    assert!(
        tokio::time::timeout(Duration::from_millis(60), &mut response)
            .await
            .is_err()
    );
    drop(held);
    let response = response.await.expect("valid test fixture");
    let elapsed = started.elapsed();
    let local = response.redirect_admission_wait();

    assert!(local >= Duration::from_millis(50));
    assert!(local < elapsed);
    let origin_elapsed = response.origin_elapsed(elapsed);
    assert!(origin_elapsed >= Duration::from_millis(30));
    assert_eq!(origin_elapsed + local, elapsed);
}
