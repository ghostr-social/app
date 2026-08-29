mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::hls_start::wait_for_start;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams};

#[tokio::test]
async fn progressive_to_hls_releases_the_obsolete_network_request() {
    let mut progressive = ControlledOrigin::serve(16).await;
    let hls = HlsGate::new();
    let root = serve(hls.clone()).await;
    let harness = start_harness("progressive-hls-cancel", serial_options());
    let mut item = sized_item("post", &progressive.url, 16, 1_000);
    harness
        .handle
        .update_focus(focus_now(vec![item.clone()], 0, 0));
    let old = tokio::time::timeout(Duration::from_secs(2), progressive.next())
        .await
        .expect("progressive request starts");

    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls = vec![root];
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    wait_for_start(
        &harness,
        &hls,
        "post",
        "HLS bootstrap receives the released slot",
    )
    .await;
    assert!(
        !old.send_byte().await,
        "obsolete progressive body is still live"
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
