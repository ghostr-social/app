use super::super::telemetry::FetchProgress;
use super::super::FetchFailure;
use super::support::{client, network_status};

#[tokio::test]
async fn panicking_fetch_task_preserves_usage_without_blame_or_retry() {
    let progress = FetchProgress::default();
    progress.mark_admitted(
        &client(),
        "https://panic.example/index.m3u8",
        &network_status(),
    );
    progress.add_network_bytes(37);
    let join = tokio::spawn(async { panic!("fixture fetch panic") })
        .await
        .expect_err("panicking worker join error");

    let failure = FetchFailure::task_failed(join, &progress);

    assert!(failure.is_local_terminal());
    assert_eq!(failure.task_failure_class(), Some("warp_hls_task_panicked"));
    assert_eq!(failure.network_bytes(), 37);
    assert!(failure.origin().is_some());
    assert!(!failure.records_origin_evidence());
    assert!(failure.retry_class().is_none());
    let resources = failure.actual_resources().expect("admitted request usage");
    assert_eq!(resources.network_bytes, 37);
    assert_eq!(resources.requests, 1);
}
