use super::send;
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::telemetry::MeasuredTraffic;
use crate::chunk::downloader::{ChunkSpec, DownloadTraffic};
use crate::delivery_events::{DeliveryNetworkStatus, DeliveryNetworkStatusReader};
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

struct LiveTraffic {
    network: DeliveryNetworkStatusReader,
    started: Arc<Notify>,
}

impl DownloadTraffic for LiveTraffic {
    fn current_network_class(&mut self) -> Option<NetworkClass> {
        Some(self.network.network_class())
    }

    fn request_started(&mut self) {
        self.started.notify_one();
    }

    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
}

#[tokio::test]
async fn queued_request_samples_network_class_only_after_admission() {
    let raw: Arc<dyn MediaHttpRequests> = Arc::new(MediaHttpClient::public().unwrap());
    let requests = MediaRequestExecutor::new(raw, MediaRequestLimits::try_new(1, 1).unwrap());
    let held = requests
        .get(
            "https://media.example/video.mp4",
            PreemptionAuthority::Transition,
        )
        .unwrap()
        .admit()
        .await
        .unwrap();
    let wifi = DeliveryNetworkStatus::new(NetworkClass::Wifi, 1);
    let network = DeliveryNetworkStatusReader::new(wifi);
    let started = Arc::new(Notify::new());
    let mut traffic = LiveTraffic {
        network: network.clone(),
        started: started.clone(),
    };
    let mut measured = MeasuredTraffic::new(&mut traffic, NetworkClass::Wifi);
    let (handle, token) = cancel_pair();
    let spec = spec(&requests);
    {
        let future = send(&spec, &token, &mut measured);
        tokio::pin!(future);
        expect_queued(&mut future).await;
        network.update(DeliveryNetworkStatus::new(NetworkClass::Cellular, 2));
        let stopper = tokio::spawn(async move {
            started.notified().await;
            handle.cancel();
        });
        drop(held);
        let _ = future.await;
        stopper.await.unwrap();
    }
    assert_eq!(
        measured.measurements().network_class(),
        NetworkClass::Cellular
    );
}

async fn expect_queued<F>(future: &mut std::pin::Pin<&mut F>)
where
    F: std::future::Future<Output = anyhow::Result<super::Opened>>,
{
    tokio::select! {
        biased;
        _ = future => panic!("request was not queued"),
        _ = tokio::task::yield_now() => {}
    }
}

fn spec(requests: &MediaRequestExecutor) -> ChunkSpec<'_> {
    ChunkSpec {
        requests,
        url: "https://media.example/video.mp4",
        request: RetrievalRequest::FetchRange {
            bytes: ByteRange::new(0, 1),
            promotion: None,
        },
        priority: PreemptionAuthority::Transition,
        continuation: None,
        timeouts: TransferTimeouts::default(),
    }
}
