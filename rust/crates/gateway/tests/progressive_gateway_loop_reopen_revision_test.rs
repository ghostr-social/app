mod gateway_fixture;
#[path = "progressive_gateway_loop_reopen_revision_test/support/mod.rs"]
mod support;

use axum::http::{header::RANGE, StatusCode};
use core::time::Duration;

#[tokio::test]
async fn used_capability_reopens_after_same_durable_generation_completes() {
    let fixture = support::seeded_harness().await;
    let before = fixture.snapshot().await;
    let capability = fixture.harness.issue_video_asset("clip").await;
    let (url, server) = support::serve(&fixture.harness, capability.as_str()).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client");
    let first = client
        .get(&url)
        .header(RANGE, "bytes=0-1")
        .send()
        .await
        .expect("first open");
    assert_eq!(first.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        first.bytes().await.expect("first body"),
        &support::BODY[..2]
    );

    fixture.commit_durable_full_body().await;
    let after = fixture.snapshot().await;
    assert_eq!(after.binding(), before.binding());
    let refreshed = fixture.harness.issue_video_asset("clip").await;
    let replay = client.get(&url).send().await.expect("loop reopen");

    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(replay.bytes().await.expect("loop body"), support::BODY);
    assert_eq!(refreshed, capability);
    assert_eq!(after.revision(), before.revision());
    server.abort();
    fixture.cleanup();
}
