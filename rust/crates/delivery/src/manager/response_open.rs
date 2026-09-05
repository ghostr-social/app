use crate::chunk::downloader::{OpenedResponse, ResponseAdmission, ResponseObservation};
use crate::manager::inflight::ChunkAttempt;
use crate::manager::DeliveryWorker;
use core::time::Duration;
use ghostr_net::media_log_identity::MediaLogIdentity;
use ghostr_partial_store::partial_range_store::{ResponseOpenResult, StoreAction};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Instant};

const RESPONSE_OPEN_CAPACITY: usize = 16;

mod evidence;
mod identity;
mod metrics;
mod promotion;
mod sizing;
mod store;
use sizing::response_bytes;

#[derive(Clone)]
pub(crate) struct ResponseOpener {
    sender: mpsc::Sender<ResponseOpenRequest>,
    timeout: Duration,
}

pub(crate) struct ResponseOpenRequest {
    pub attempt: ChunkAttempt,
    pub action: StoreAction,
    pub response: OpenedResponse,
    pub opened_at: Instant,
    pub reply: oneshot::Sender<ResponseAdmission>,
}

pub(crate) type ResponseOpenReceiver = mpsc::Receiver<ResponseOpenRequest>;

pub(crate) fn channel(timeout: Duration) -> (ResponseOpener, ResponseOpenReceiver) {
    let (sender, receiver) = mpsc::channel(RESPONSE_OPEN_CAPACITY);
    (ResponseOpener { sender, timeout }, receiver)
}

impl ResponseOpener {
    pub(super) async fn authorize(
        &self,
        attempt: ChunkAttempt,
        action: StoreAction,
        response: OpenedResponse,
    ) -> ResponseAdmission {
        let (reply, answer) = oneshot::channel();
        let request = ResponseOpenRequest {
            attempt,
            action,
            response,
            opened_at: Instant::now(),
            reply,
        };
        let exchange = async {
            self.sender.send(request).await.ok()?;
            answer.await.ok()
        };
        timeout(self.timeout, exchange)
            .await
            .ok()
            .flatten()
            .unwrap_or(ResponseAdmission::Reject)
    }
}

impl DeliveryWorker {
    pub(super) async fn apply_response_open(&mut self, request: ResponseOpenRequest) {
        let ResponseOpenRequest {
            attempt,
            action,
            response,
            opened_at,
            reply,
        } = request;
        if reply.is_closed() || opened_at.elapsed() > self.ctx.timeouts.idle {
            self.downloads.reject_response(&attempt);
            return;
        }
        let admission = self
            .decide_response_open(&attempt, &action, &response)
            .await;
        if reply.send(admission).is_err() && admission == ResponseAdmission::Proceed {
            self.downloads.reject_response(&attempt);
            self.ctx.store.release_action(&action).await;
        }
    }

    async fn decide_response_open(
        &mut self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
    ) -> ResponseAdmission {
        if !self
            .response_identity_current(attempt, action, response)
            .await
        {
            return ResponseAdmission::Reject;
        }
        let admission = match self.adopt_response_generation(attempt, response).await {
            Ok(Some(admission)) => admission,
            Ok(None) => {
                self.downloads.reject_response(attempt);
                return ResponseAdmission::Reject;
            }
            Err(error) => {
                self.reject_open_error(attempt, &error);
                return ResponseAdmission::Reject;
            }
        };
        if let Err(error) = self.resize_response_action(action, response).await {
            self.reject_open_error(attempt, &error);
            return ResponseAdmission::Reject;
        }
        match self
            .open_store_response(attempt, action, response, admission)
            .await
        {
            Ok(ResponseOpenResult::Opened) => {
                self.timelines
                    .observe_index_source(attempt.identity(), response);
                self.downloads
                    .observe_response(attempt, response.observation());
                ResponseAdmission::Proceed
            }
            Ok(ResponseOpenResult::RequiresIndependentObject) => {
                self.record_independent_object(attempt);
                self.downloads.reject_response(attempt);
                ResponseAdmission::Reject
            }
            Ok(ResponseOpenResult::Stale) => self.reject_stale_response(),
            Err(error) => {
                self.reject_open_error(attempt, &error);
                ResponseAdmission::Reject
            }
        }
    }

    async fn resize_response_action(
        &self,
        action: &StoreAction,
        response: &OpenedResponse,
    ) -> anyhow::Result<()> {
        self.ctx
            .store
            .resize_action(action, response_bytes(response.observation()))
            .await
    }

    fn reject_open_error(&mut self, attempt: &ChunkAttempt, error: &anyhow::Error) {
        self.downloads.reject_response(attempt);
        if !self.absorb_store_pressure(attempt.identity().post(), error) {
            log::warn!(
                "Could not authorize video response for {}",
                MediaLogIdentity::from_url(attempt.identity().source().as_str())
            );
        }
    }

    pub(super) fn record_independent_object(&mut self, attempt: &ChunkAttempt) {
        self.independent_objects.record(attempt.identity().clone());
    }
}
