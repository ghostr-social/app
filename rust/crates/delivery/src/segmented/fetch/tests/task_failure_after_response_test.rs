use super::super::telemetry::FetchProgress;
use super::super::FetchFailure;
use super::support::{client, network_status};

#[tokio::test]
async fn local_task_failure_preserves_completed_response_evidence() {
    let progress = FetchProgress::default();
    progress.mark_admitted(
        &client(),
        "https://panic.example/index.m3u8",
        &network_status(),
    );
    progress.add_network_bytes(37);
    progress.finish_response();
    let join = tokio::spawn(async { panic!("fixture preparation panic") })
        .await
        .expect_err("panicking worker join error");

    let failure = FetchFailure::task_failed(join, &progress);

    assert!(failure.response_completed());
}
