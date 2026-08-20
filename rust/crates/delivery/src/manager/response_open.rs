use crate::chunk::downloader::{OpenedResponse, ResponseAdmission, ResponseObservation};
use crate::manager::inflight::ChunkAttempt;
use crate::manager::time::unix_time_ms;
use crate::manager::DeliveryWorker;
use ghostr_net::media_log_identity::MediaLogIdentity;
use ghostr_partial_store::partial_range_store::{ResponseOpenResult, StoreAction};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Instant};

const RESPONSE_OPEN_CAPACITY: usize = 16;

mod evidence;
mod metrics;

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
    pub opened_at_ms: u64,
    pub reply: oneshot::Sender<ResponseAdmission>,
}

pub(crate) type ResponseOpenReceiver = mpsc::Receiver<ResponseOpenRequest>;

pub(crate) fn channel(timeout: Duration) -> (ResponseOpener, ResponseOpenReceiver) {
    let (sender, receiver) = mpsc::channel(RESPONSE_OPEN_CAPACITY);
    (ResponseOpener { sender, timeout }, receiver)
}

impl ResponseOpener {
    pub(crate) async fn authorize(
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
            opened_at_ms: unix_time_ms(),
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
    pub(crate) async fn apply_response_open(&mut self, request: ResponseOpenRequest) {
        let ResponseOpenRequest {
            attempt,
            action,
            response,
            opened_at,
            opened_at_ms,
            reply,
        } = request;
        if reply.is_closed() || opened_at.elapsed() > self.ctx.timeouts.idle {
            self.downloads.reject_response(&attempt);
            return;
        }
        let admission = self
            .decide_response_open(&attempt, &action, &response, opened_at_ms)
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
        opened_at_ms: u64,
    ) -> ResponseAdmission {
        if !self
            .downloads
            .authorizes_response(attempt, action, response, opened_at_ms)
        {
            return ResponseAdmission::Reject;
        }
        self.learn_opened_response(attempt, response, opened_at_ms);
        if let Err(error) = self.resize_response_action(action, response).await {
            self.reject_open_error(attempt, &error);
            return ResponseAdmission::Reject;
        }
        match self.open_store_response(attempt, action, response).await {
            Ok(ResponseOpenResult::Opened) => {
                self.downloads
                    .observe_response(attempt, response.observation());
                ResponseAdmission::Proceed
            }
            Ok(ResponseOpenResult::RequiresIndependentObject) => {
                self.record_independent_object(attempt).await;
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

    async fn open_store_response(
        &self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
    ) -> anyhow::Result<ResponseOpenResult> {
        match response.mode() {
            crate::chunk::sink::ResponseWriteMode::Sparse => {
                let Some(generation) = response.generation().cloned() else {
                    return Ok(ResponseOpenResult::RequiresIndependentObject);
                };
                let ResponseObservation::Partial { range, .. } = response.observation() else {
                    return Ok(ResponseOpenResult::RequiresIndependentObject);
                };
                self.ctx
                    .store
                    .open_sparse_response(attempt.identity(), action, generation, range)
                    .await
            }
            crate::chunk::sink::ResponseWriteMode::SingleResponse(contract) => {
                self.ctx
                    .store
                    .open_single_response_for_action(attempt.identity(), action, contract)
                    .await
            }
        }
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

    async fn record_independent_object(&mut self, attempt: &ChunkAttempt) {
        let identity = attempt.identity();
        let Ok(snapshot) = self
            .ctx
            .store
            .media_snapshot(identity.post().as_str())
            .await
        else {
            return;
        };
        self.independent_objects.record(
            identity.post().clone(),
            identity.source().as_str().to_owned(),
            snapshot.revision(),
        );
    }
}

fn response_bytes(response: ResponseObservation) -> u64 {
    match response {
        ResponseObservation::Partial { range, .. } => range.len(),
        ResponseObservation::Body { request, .. } => request.reserved_network_bytes(),
        ResponseObservation::Ignored { .. } => 0,
    }
}
