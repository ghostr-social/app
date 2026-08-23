use super::{InternalEvent, ObservedResponse, TransferContext, TransferEvent};
use crate::chunk::downloader::{OpenedResponse, ResponseAdmission};
use crate::chunk::traffic::ChunkTraffic;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::response_open::ResponseOpener;
use crate::manager::traffic::{TrafficPublisher, TransferKey};
use ghostr_partial_store::partial_range_store::StoreAction;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

pub(super) struct TransferTraffic {
    attempt: ChunkAttempt,
    transfer: TransferKey,
    host: Option<String>,
    publisher: TrafficPublisher,
    events: UnboundedSender<InternalEvent>,
    responses: ResponseOpener,
    store_action: StoreAction,
    network_status: crate::delivery_events::DeliveryNetworkStatusReader,
    opened: bool,
}

impl TransferTraffic {
    pub(super) fn new(
        attempt: &ChunkAttempt,
        ctx: &TransferContext,
        url: &str,
        store_action: StoreAction,
    ) -> Self {
        Self {
            attempt: attempt.clone(),
            transfer: TransferKey::new(attempt.id().value()),
            host: ghostr_engine::host_stats::host_of(url),
            publisher: ctx.traffic.clone(),
            events: ctx.events.clone(),
            responses: ctx.responses.clone(),
            store_action,
            network_status: ctx.network_status.clone(),
            opened: false,
        }
    }
}

impl ChunkTraffic for TransferTraffic {
    fn current_network_class(&mut self) -> Option<ghostr_engine::origin_model::NetworkClass> {
        Some(self.network_status.network_class())
    }

    fn opened(&mut self, ttfb: Duration) {
        let Some(host) = self.host.take() else {
            return;
        };
        self.opened = self
            .publisher
            .opened(self.transfer, host, ttfb, Instant::now());
    }

    fn wrote(&mut self, bytes: u64) {
        if self.opened {
            self.publisher
                .progress(self.transfer, bytes, Instant::now());
        }
    }

    fn response_observed(&mut self, response: OpenedResponse) {
        let observed = ObservedResponse::at_network_boundary(self.attempt.clone(), response);
        let event = TransferEvent::ResponseObserved(Box::new(observed));
        let _ = self.events.send(InternalEvent::Transfer(event));
    }

    fn authorize_response<'a>(
        &'a mut self,
        response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        let attempt = self.attempt.clone();
        let action = self.store_action.clone();
        let responses = self.responses.clone();
        Box::pin(async move { Ok(responses.authorize(attempt, action, response).await) })
    }
}

impl Drop for TransferTraffic {
    fn drop(&mut self) {
        if self.opened {
            self.publisher.closed(self.transfer, Instant::now());
        }
    }
}
