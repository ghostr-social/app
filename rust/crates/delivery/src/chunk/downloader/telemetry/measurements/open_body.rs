use crate::chunk::downloader::ResponseObservation;
use ghostr_engine::adaptive::{RetrievalRequest, WholeFetchReason};
use tokio::time::Instant;

#[derive(Clone, Debug)]
pub(in crate::chunk::downloader) struct OpenBodyMeasurement {
    planned_bytes: u64,
    byte_baseline: u64,
    admitted_at: Instant,
}

impl OpenBodyMeasurement {
    pub(in crate::chunk::downloader) fn accepted(planned_bytes: u64, byte_baseline: u64) -> Self {
        Self {
            planned_bytes,
            byte_baseline,
            admitted_at: Instant::now(),
        }
    }

    pub(in crate::chunk::downloader) const fn planned_bytes(&self) -> u64 {
        self.planned_bytes
    }

    pub(in crate::chunk::downloader) const fn received_bytes(&self, total: u64) -> u64 {
        total.saturating_sub(self.byte_baseline)
    }

    pub(in crate::chunk::downloader) fn elapsed(&self) -> core::time::Duration {
        self.admitted_at.elapsed()
    }
}

pub(super) fn promoted_body_bytes(observation: ResponseObservation) -> Option<u64> {
    let ResponseObservation::Body {
        request: RetrievalRequest::FetchWhole { contract, reason },
        promoted: true,
        ..
    } = observation
    else {
        return None;
    };
    (reason == WholeFetchReason::PromotedResponse).then_some(contract.maximum_bytes())
}
