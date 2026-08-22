use super::super::{fetch_stage, FetchFailure, FetchedObject, StagedFetch};
use super::support::{client, immediate_asset, immediate_failure, network_status};
use crate::manager::time::unix_time_ms;
use ghostr_engine::adaptive::{HlsBootstrapStage, PreemptionAuthority};
use ghostr_engine::origin_model::ErrorReason;
use ghostr_engine::origin_model::NetworkClass;
use std::time::{Duration, Instant};

#[tokio::test]
async fn staged_success_excludes_gate_queue_and_keeps_network_class() {
    let (url, server) = immediate_asset().await;
    let (wall, result) = queued_fetch(url).await;
    let object = result.unwrap_or_else(|error| panic!("staged object: {error}"));

    assert_queue_excluded(wall, object.telemetry.elapsed);
    assert_eq!(object.telemetry.concurrency, 1);
    assert_eq!(object.telemetry.network_class, NetworkClass::Cellular);
    server.await.unwrap();
}

#[tokio::test]
async fn staged_header_failure_excludes_shared_gate_queue_wait() {
    let (url, server) = immediate_failure().await;
    let (wall, result) = queued_fetch(url).await;
    let failure = match result {
        Ok(_) => panic!("origin must reject request"),
        Err(failure) => failure,
    };
    let telemetry = failure.origin().expect("request was admitted");

    assert_queue_excluded(wall, telemetry.elapsed);
    assert_eq!(telemetry.concurrency, 1);
    assert_eq!(telemetry.network_class, NetworkClass::Cellular);
    assert_eq!(failure.reason(), ErrorReason::Http5xx);
    server.await.unwrap();
}

async fn queued_fetch(url: String) -> (Duration, Result<FetchedObject, FetchFailure>) {
    let requests = client();
    let held = requests
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap();
    let staged_requests = requests.clone();
    let wall_started = Instant::now();
    let task = tokio::spawn(async move {
        let network = network_status();
        fetch_stage(StagedFetch {
            requests: &staged_requests,
            stage: HlsBootstrapStage::FirstSegment,
            url: &url,
            maximum_bytes: 256 * 1024,
            continuation: None,
            priority: PreemptionAuthority::PlaybackCritical,
            committed_until_ms: unix_time_ms() + 1_000,
            network_status: &network,
            cancellation: None,
            traffic: None,
        })
        .await
    });
    tokio::time::sleep(Duration::from_millis(250)).await;
    drop(held);
    (wall_started.elapsed(), task.await.unwrap())
}

fn assert_queue_excluded(wall: Duration, origin: Duration) {
    assert!(wall >= Duration::from_millis(250));
    assert!(wall.saturating_sub(origin) >= Duration::from_millis(150));
}
