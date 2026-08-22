use super::super::{fetch_stage, StagedFetch};
use super::support::{client, delayed_asset, immediate_asset, network_status};
use crate::manager::time::unix_time_ms;
use ghostr_engine::adaptive::{HlsBootstrapStage, PreemptionAuthority};
use ghostr_engine::origin_model::ErrorReason;
use std::time::Duration;

#[tokio::test]
async fn admitted_stage_may_complete_after_its_ownership_fence() {
    let (url, server) = delayed_asset(Duration::from_millis(300)).await;
    let requests = client();
    let network = network_status();
    let committed_until_ms = unix_time_ms() + 200;
    let object = fetch_stage(input(&requests, &network, &url, committed_until_ms))
        .await
        .unwrap_or_else(|error| panic!("bounded transfer must finish: {error}"));

    assert_eq!(object.body.as_ref(), b"x");
    assert!(unix_time_ms() > committed_until_ms);
    server.await.unwrap();
}

#[tokio::test]
async fn expired_stage_is_rejected_before_origin_contact() {
    let (url, server) = delayed_asset(Duration::ZERO).await;
    let requests = client();
    let network = network_status();
    let result = fetch_stage(input(
        &requests,
        &network,
        &url,
        unix_time_ms().saturating_sub(1),
    ))
    .await;
    let error = match result {
        Ok(_) => panic!("stale ownership cannot launch"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("expired before launch"));
    assert!(error.origin().is_none());
    server.abort();
}

#[tokio::test]
async fn gate_admission_after_the_ownership_fence_never_contacts_the_origin() {
    let (url, server) = immediate_asset().await;
    let requests = client();
    let held = requests
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap();
    let committed_until_ms = unix_time_ms() + 40;
    let queued_requests = requests.clone();
    let queued_url = url.clone();
    let task = tokio::spawn(async move {
        let network = network_status();
        fetch_stage(input(
            &queued_requests,
            &network,
            &queued_url,
            committed_until_ms,
        ))
        .await
    });
    tokio::time::sleep(Duration::from_millis(80)).await;
    drop(held);
    let error = match task.await.unwrap() {
        Ok(_) => panic!("expired queued stage must fail"),
        Err(error) => error,
    };

    assert_eq!(error.reason(), ErrorReason::Policy);
    assert!(error.origin().is_none());
    assert!(!server.is_finished(), "expired stage contacted the origin");
    server.abort();
}

fn input<'a>(
    requests: &'a ghostr_net::media_request_executor::MediaRequestExecutor,
    network: &'a crate::delivery_events::DeliveryNetworkStatusReader,
    url: &'a str,
    committed_until_ms: u64,
) -> StagedFetch<'a> {
    StagedFetch {
        requests,
        stage: HlsBootstrapStage::FirstSegment,
        url,
        maximum_bytes: 256 * 1024,
        continuation: None,
        priority: PreemptionAuthority::PlaybackCritical,
        committed_until_ms,
        network_status: network,
        cancellation: None,
        traffic: None,
    }
}
