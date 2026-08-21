#![cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]

mod gateway_fixture;
#[path = "gateway_shared_request_gate_fixture/mod.rs"]
mod shared_gate_fixture;

#[tokio::test]
async fn progressive_body_and_gateway_hls_share_the_global_request_gate() {
    shared_gate_fixture::SharedGateScenario::start()
        .await
        .exercise()
        .await;
}
