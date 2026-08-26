use super::super::axiom_test_support::{fetch, FetchInput};
use super::super::{FetchSpec, SegmentedTraffic};
use super::support::{client, immediate_asset};
use crate::delivery_events::{DeliveryNetworkStatus, DeliveryNetworkStatusReader};
use crate::manager::traffic::{channel, TrafficEvent, TrafficPublisher};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::ActionId;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn staged_hls_bytes_enter_the_shared_measured_traffic_stream() {
    let (url, server) = immediate_asset().await;
    let url = url::Url::parse(&url).expect("valid test fixture");
    let (events, _receiver) = tokio::sync::mpsc::unbounded_channel();
    let (publisher, inbox) = channel(events, 8);

    let network = DeliveryNetworkStatusReader::new(DeliveryNetworkStatus::unavailable());
    let object = fetch(&client(), input(&url, publisher), &network, None)
        .await
        .unwrap_or_else(|error| panic!("measured HLS fetch: {error}"));
    let batch = inbox.drain(tokio::time::Instant::now());

    assert_eq!(object.body.len(), 1);
    assert!(matches!(
        batch.events().first(),
        Some(TrafficEvent::Opened { .. })
    ));
    assert!(matches!(
        batch.events().last(),
        Some(TrafficEvent::Closed { .. })
    ));
    assert_eq!(
        batch
            .events()
            .iter()
            .filter_map(|event| match event {
                TrafficEvent::Progress { bytes, .. } => Some(*bytes),
                _ => None,
            })
            .sum::<u64>(),
        1
    );
    server.await.expect("valid test fixture");
}

fn input(url: &url::Url, publisher: TrafficPublisher) -> FetchInput<'_> {
    FetchInput {
        spec: FetchSpec {
            url: url.as_str(),
            limit: MAX_HLS_ASSET_BYTES,
            object_limit: MAX_HLS_ASSET_BYTES as u64,
            object: Default::default(),
            timeouts: HlsTransferTimeouts::default(),
            priority: PreemptionAuthority::Transition,
            admission_fence: None,
        },
        traffic: Some(SegmentedTraffic::new(ActionId::new(17), publisher)),
    }
}
