use super::{open, Opened};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{OpenedResponse, ResponseAdmission};
use crate::chunk::traffic::ChunkTraffic;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use std::sync::Arc;

struct IgnoreTraffic;

impl ChunkTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
    fn response_observed(&mut self, _response: OpenedResponse) {}
    fn authorize_response<'a>(
        &'a mut self,
        _response: OpenedResponse,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>,
    > {
        Box::pin(async { Ok(ResponseAdmission::Proceed) })
    }
}

#[tokio::test]
async fn cancellation_before_the_send_future_is_polled_is_not_a_started_request() {
    let raw: Arc<dyn MediaHttpRequests> =
        Arc::new(MediaHttpClient::public().expect("valid test fixture"));
    let requests = MediaRequestExecutor::new(
        raw,
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let admitted = requests
        .get(
            "https://media.example/video.mp4",
            PreemptionAuthority::Transition,
        )
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture");
    let (handle, token) = cancel_pair();
    handle.cancel();

    let result = open(admitted, &token, Duration::from_secs(1), &mut IgnoreTraffic).await;

    assert!(matches!(
        result.expect("valid test fixture"),
        Opened::CancelledBeforeRequest
    ));
}
