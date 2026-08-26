mod request_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    MediaRequestAdmissionTimeout, MediaRequestExecutor, MediaRequestLimits,
};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};

#[tokio::test]
async fn admission_timeout_is_distinct_from_an_origin_failure() {
    let mut held = HeldOrigin::serve().await;
    let requests = executor();
    let active = occupy(&requests, &mut held).await;
    let error = admission_error(&requests, &held.url).await;

    assert!(error.is::<MediaRequestAdmissionTimeout>());
    held.expect_quiet().await;
    held.release_one();
    drop(active);
}

fn executor() -> MediaRequestExecutor {
    MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    )
}

async fn occupy(
    requests: &MediaRequestExecutor,
    held: &mut HeldOrigin,
) -> ghostr_net::media_request_executor::MediaResponse {
    let active = requests
        .get(&held.url, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture");
    held.expect_hit().await;
    active
}

async fn admission_error(requests: &MediaRequestExecutor, url: &str) -> anyhow::Error {
    requests
        .get(url, PreemptionAuthority::PlaybackCritical)
        .expect("valid test fixture")
        .admit_for(Duration::from_millis(20))
        .await
        .err()
        .expect("occupied gate must bound admission wait")
}
