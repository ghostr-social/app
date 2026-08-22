use crate::hls::asset_response::{AssetBodyContract, AssetResponseEnvelope};
use anyhow::{ensure, Context, Result};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use bytes::Bytes;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{MediaRequest, MediaResponse};
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use tokio::time::{timeout_at, Instant};

mod proxy;

#[cfg(test)]
#[path = "transfer/header_timeout_test.rs"]
mod header_timeout_test;
#[cfg(test)]
#[path = "transfer/paused_body_total_timeout_test.rs"]
mod paused_body_total_timeout_test;
#[cfg(test)]
#[path = "transfer/test_fixture.rs"]
mod test_fixture;
#[cfg(test)]
#[path = "transfer/total_admission_timeout_test.rs"]
mod total_admission_timeout_test;
#[cfg(test)]
#[path = "transfer/total_header_timeout_test.rs"]
mod total_header_timeout_test;

pub(super) struct HlsTransfer {
    response: MediaResponse,
    idle: std::time::Duration,
    total_deadline: Instant,
}

impl HlsTransfer {
    pub(super) async fn open(request: MediaRequest, timeouts: HlsTransferTimeouts) -> Result<Self> {
        let deadline = Instant::now() + timeouts.total;
        Self::open_at(request, timeouts, deadline).await
    }

    pub(super) async fn open_at(
        request: MediaRequest,
        timeouts: HlsTransferTimeouts,
        total_deadline: Instant,
    ) -> Result<Self> {
        let admitted = timeout_at(total_deadline, request.admit())
            .await
            .context("HLS object transfer timed out")??;
        let header_deadline = total_deadline.min(Instant::now() + timeouts.headers);
        let timeout_context = header_timeout_context(header_deadline, total_deadline);
        let sending = admitted.send_with_redirect_deadline(header_deadline);
        let response = timeout_at(header_deadline, sending)
            .await
            .context(timeout_context)??;
        validate_response_headers(response.headers())?;
        require_identity_encoding(response.headers()).context("encoded HLS upstream response")?;
        Ok(Self {
            response,
            idle: timeouts.idle,
            total_deadline,
        })
    }

    pub(super) fn response(&self) -> &MediaResponse {
        &self.response
    }

    pub(super) async fn read_bounded(&mut self, limit: usize) -> Result<Vec<u8>> {
        let mut body = Vec::new();
        while let Some(chunk) = self.next_chunk().await? {
            ensure!(
                body.len().saturating_add(chunk.len()) <= limit,
                "HLS object exceeds its byte limit"
            );
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    pub(super) fn into_proxy(
        self,
        envelope: AssetResponseEnvelope,
    ) -> Result<Response<Body>, StatusCode> {
        let content_type = self.response.headers().get(CONTENT_TYPE).cloned();
        let mut response = Response::builder().status(envelope.status());
        if let Some(value) = content_type {
            response = response.header(CONTENT_TYPE, value);
        }
        if let Some(length) = envelope.content_length() {
            response = response.header(CONTENT_LENGTH, length);
        }
        if let Some(range) = envelope.content_range() {
            response = response.header(CONTENT_RANGE, range);
        }
        if envelope.advertises_ranges() {
            response = response.header(ACCEPT_RANGES, "bytes");
        }
        let body = self.proxy_body(envelope.body_contract());
        response
            .body(body)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn proxy_body(self, contract: AssetBodyContract) -> Body {
        match contract {
            AssetBodyContract::Empty => Body::empty(),
            _ => proxy::body(self, contract),
        }
    }

    async fn next_chunk(&mut self) -> Result<Option<Bytes>> {
        let idle_deadline = Instant::now() + self.idle;
        let deadline = self.total_deadline.min(idle_deadline);
        let context = if deadline == self.total_deadline {
            "HLS object transfer timed out"
        } else {
            "HLS object body idle timed out"
        };
        timeout_at(deadline, self.response.chunk())
            .await
            .with_context(|| context)?
            .context("read HLS upstream body")
    }
}

fn header_timeout_context(header_deadline: Instant, total_deadline: Instant) -> &'static str {
    if header_deadline == total_deadline {
        "HLS object transfer timed out"
    } else {
        "HLS response headers timed out"
    }
}
