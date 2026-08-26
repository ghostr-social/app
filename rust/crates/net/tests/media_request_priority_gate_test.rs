mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn playback_critical_waiter_precedes_older_speculation() {
    let mut held = HeldOrigin::serve().await;
    let mut speculative = HeldOrigin::serve().await;
    let mut critical = HeldOrigin::serve().await;
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let active = open(
        requests.clone(),
        held.url.clone(),
        PreemptionAuthority::Transition,
    )
    .await;
    held.expect_hit().await;

    let later = tokio::spawn(open(
        requests.clone(),
        speculative.url.clone(),
        PreemptionAuthority::Speculative,
    ));
    speculative.expect_quiet().await;
    let urgent = tokio::spawn(open(
        requests,
        critical.url.clone(),
        PreemptionAuthority::PlaybackCritical,
    ));
    critical.expect_quiet().await;

    held.release_one();
    drop(active);
    critical.expect_hit().await;
    speculative.expect_quiet().await;
    let urgent = urgent.await.expect("valid test fixture");
    critical.release_one();
    drop(urgent);
    speculative.expect_hit().await;
    speculative.release_one();
    drop(later.await.expect("valid test fixture"));
}

async fn open(
    requests: MediaRequestExecutor,
    url: String,
    priority: PreemptionAuthority,
) -> ghostr_net::media_request_executor::MediaResponse {
    requests
        .get(&url, priority)
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
