use crate::hls::asset_response::{AssetBodyContract, AssetResponseEnvelope};
use anyhow::{ensure, Context, Result};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use bytes::Bytes;
use futures_util::stream;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::RequestBuilder;
use std::io;
use tokio::time::{timeout_at, Instant};

pub(super) struct HlsTransfer {
    response: reqwest::Response,
    idle: std::time::Duration,
    total_deadline: Instant,
}

struct AssetBodyState {
    transfer: HlsTransfer,
    contract: AssetBodyContract,
    sent: u64,
}

type BodyResult = Result<Bytes, io::Error>;
type BodyStreamState = Option<AssetBodyState>;

impl HlsTransfer {
    pub(super) async fn open(
        request: RequestBuilder,
        timeouts: HlsTransferTimeouts,
    ) -> Result<Self> {
        let deadline = Instant::now() + timeouts.total;
        Self::open_at(request, timeouts, deadline).await
    }

    pub(super) async fn open_at(
        request: RequestBuilder,
        timeouts: HlsTransferTimeouts,
        total_deadline: Instant,
    ) -> Result<Self> {
        let header_deadline = total_deadline.min(Instant::now() + timeouts.headers);
        let response = timeout_at(header_deadline, request.send())
            .await
            .context("HLS response headers timed out")??;
        validate_response_headers(response.headers())?;
        require_identity_encoding(response.headers()).context("encoded HLS upstream response")?;
        Ok(Self {
            response,
            idle: timeouts.idle,
            total_deadline,
        })
    }

    pub(super) fn response(&self) -> &reqwest::Response {
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
            _ => timed_body(self, contract),
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

fn timed_body(transfer: HlsTransfer, contract: AssetBodyContract) -> Body {
    let state = AssetBodyState {
        transfer,
        contract,
        sent: 0,
    };
    Body::from_stream(stream::unfold(Some(state), next_body_item))
}

async fn next_body_item(state: BodyStreamState) -> Option<(BodyResult, BodyStreamState)> {
    let mut state = state?;
    match state.transfer.next_chunk().await {
        Ok(Some(chunk)) => Some(state.forward(chunk)),
        Ok(None) => state.finish(),
        Err(error) => Some(failed(io::ErrorKind::TimedOut, error.to_string())),
    }
}

impl AssetBodyState {
    fn forward(mut self, chunk: Bytes) -> (BodyResult, BodyStreamState) {
        let Some(total) = self.contract.checked_total(self.sent, chunk.len()) else {
            return failed(
                io::ErrorKind::InvalidData,
                "HLS body exceeds its extent".to_owned(),
            );
        };
        self.sent = total;
        (Ok(chunk), Some(self))
    }

    fn finish(self) -> Option<(BodyResult, BodyStreamState)> {
        (!self.contract.complete(self.sent)).then(|| {
            failed(
                io::ErrorKind::UnexpectedEof,
                "HLS body ended early".to_owned(),
            )
        })
    }
}

fn failed(kind: io::ErrorKind, message: String) -> (BodyResult, BodyStreamState) {
    (Err(io::Error::new(kind, message)), None)
}
