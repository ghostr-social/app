mod redirect_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use redirect_gate_fixture::target::TargetOrigin;
use redirect_gate_fixture::{redirect_origin, OneHopClient};
use reqwest::header::{HeaderValue, AUTHORIZATION, COOKIE, IF_RANGE, PROXY_AUTHORIZATION, RANGE};

#[tokio::test]
async fn each_redirect_hop_waits_for_its_actual_authority_permit() {
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
    assert!(target.hit().await.starts_with("GET /held "));

    let redirected = executor
        .get(&start, PreemptionAuthority::PlaybackCritical)
        .expect("valid test fixture")
        .header(RANGE, HeaderValue::from_static("bytes=4-9"))
        .header(IF_RANGE, HeaderValue::from_static("\"v1\""))
        .header(AUTHORIZATION, HeaderValue::from_static("Bearer private"))
        .header(COOKIE, HeaderValue::from_static("session=private"))
        .header(
            PROXY_AUTHORIZATION,
            HeaderValue::from_static("Basic private"),
        )
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        );
    tokio::pin!(redirected);
    assert!(
        tokio::time::timeout(Duration::from_millis(75), &mut redirected)
            .await
            .is_err()
    );
    target.quiet().await;

    drop(held);
    let response = redirected.await.expect("valid test fixture");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let request = target.hit().await.to_ascii_lowercase();
    assert!(request.contains("range: bytes=4-9"));
    assert!(request.contains("if-range: \"v1\""));
    assert!(!request.contains("authorization:"));
    assert!(!request.contains("cookie:"));
    assert!(!request.contains("proxy-authorization:"));

    let next = executor
        .get(&target.redirected_url, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit();
    tokio::pin!(next);
    assert!(tokio::time::timeout(Duration::from_millis(75), &mut next)
        .await
        .is_err());
    drop(response);
    let admitted = next.await.expect("valid test fixture");
    let _ = admitted
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture");
    assert!(target.hit().await.starts_with("GET /redirected "));
}
