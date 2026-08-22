use crate::manager::traffic::{TrafficPublisher, TransferKey};
use ghostr_engine::ActionId;
use ghostr_net::media_request_executor::MediaResponse;
use std::time::Duration;
use tokio::time::Instant;

pub(in crate::segmented) struct SegmentedTraffic {
    publisher: TrafficPublisher,
    transfer: TransferKey,
    opened: bool,
}

impl SegmentedTraffic {
    pub(in crate::segmented) fn new(action: ActionId, publisher: TrafficPublisher) -> Self {
        Self {
            publisher,
            transfer: TransferKey::new(action.value()),
            opened: false,
        }
    }

    pub(in crate::segmented) fn opened(&mut self, response: &MediaResponse, ttfb: Duration) {
        if self.opened {
            return;
        }
        let Some(host) = ghostr_engine::host_stats::host_of(response.url().as_str()) else {
            return;
        };
        self.opened = self
            .publisher
            .opened(self.transfer, host, ttfb, Instant::now());
    }

    pub(in crate::segmented) fn progress(&self, bytes: u64) {
        if self.opened {
            self.publisher
                .progress(self.transfer, bytes, Instant::now());
        }
    }
}

impl Drop for SegmentedTraffic {
    fn drop(&mut self) {
        if self.opened {
            self.publisher.closed(self.transfer, Instant::now());
        }
    }
}
