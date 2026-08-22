mod delivery_fixture;
mod hls_terminal_wait;
use axum::http::StatusCode;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::hls_recovery::{serve as serve_script, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

#[tokio::test]
async fn failed_first_hls_source_switches_to_the_healthy_mirror() {
    let primary = HlsScript::new("root", [StatusCode::NOT_FOUND]);
    let invalid = serve_script(primary.clone()).await;
    let gate = HlsGate::new();
    let valid = serve(gate.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.base = Duration::from_secs(5);
    options.tuning.retry.max = Duration::from_secs(5);
    let harness = start_harness("hls-mirror-fallback", options);
    let mut item = sized_item("stream", &invalid, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls.push(valid);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    tokio::time::timeout(Duration::from_secs(2), gate.started.acquire())
        .await
        .expect("healthy mirror starts before the failed source backoff")
        .unwrap()
        .forget();
    gate.release.add_permits(1);
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;
    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    assert_eq!(primary.paths(), vec!["root"]);
    assert_eq!(gate.hits(), vec!["root", "child", "init", "segment"]);

    std::fs::remove_dir_all(&harness.root).ok();
}
