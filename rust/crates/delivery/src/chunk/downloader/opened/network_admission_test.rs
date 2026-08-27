use super::send;
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::telemetry::MeasuredTraffic;
use crate::chunk::downloader::DownloadTraffic;
use crate::delivery_events::{DeliveryNetworkStatus, DeliveryNetworkStatusReader};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use std::sync::Arc;
use tokio::sync::Notify;

#[path = "network_admission_test/fixture.rs"]
mod fixture;
use fixture::spec;

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
    let raw: Arc<dyn MediaHttpRequests> =
        Arc::new(MediaHttpClient::public().expect("valid test fixture"));
    let requests = MediaRequestExecutor::new(
        raw,
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let held = requests
        .get(
            "https://media.example/video.mp4",
            PreemptionAuthority::Transition,
        )
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture");
    let wifi = DeliveryNetworkStatus::new(NetworkClass::Wifi, 1);
    let network = DeliveryNetworkStatusReader::new(wifi);
    let started = Arc::new(Notify::new());
    let mut traffic = LiveTraffic {
        network: network.clone(),
        started: std::sync::Arc::clone(&started),
    };
    let (handle, token) = cancel_pair();
    let spec = spec(&requests);
    let mut measured = MeasuredTraffic::new(&mut traffic, NetworkClass::Wifi, spec.attempt_profile);
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
        stopper.await.expect("valid test fixture");
    }
    let context = measured
        .measurements()
        .attempt_context()
        .expect("started request context");
    assert_eq!(context.request_context().network, NetworkClass::Cellular);
}

async fn expect_queued<F>(future: &mut core::pin::Pin<&mut F>)
where
    F: core::future::Future<Output = anyhow::Result<super::Opened>>,
{
    tokio::select! {
        biased;
        _ = future => panic!("request was not queued"),
        () = tokio::task::yield_now() => {}
    }
}
