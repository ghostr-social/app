use anyhow::{ensure, Context, Result};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use bytes::Bytes;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::RequestBuilder;
use std::io;
use tokio::sync::mpsc;
use tokio::time::{timeout_at, Instant};
use tokio_stream::wrappers::ReceiverStream;

pub(super) struct HlsTransfer {
    response: reqwest::Response,
    idle: std::time::Duration,
    total_deadline: Instant,
}

impl HlsTransfer {
    pub(super) async fn open(
        request: RequestBuilder,
        timeouts: HlsTransferTimeouts,
    ) -> Result<Self> {
        let total_deadline = Instant::now() + timeouts.total;
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

    pub(super) fn into_proxy(self) -> Result<Response<Body>, StatusCode> {
        let status = self.response.status();
        let headers = self.response.headers().clone();
        let mut response = Response::builder().status(status);
        for name in [CONTENT_TYPE, CONTENT_LENGTH, CONTENT_RANGE, ACCEPT_RANGES] {
            if let Some(value) = headers.get(&name) {
                response = response.header(name, value);
            }
        }
        response
            .body(timed_body(self))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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

fn timed_body(mut transfer: HlsTransfer) -> Body {
    let (sender, receiver) = mpsc::channel(1);
    tokio::spawn(async move {
        loop {
            match transfer.next_chunk().await {
                Ok(Some(chunk)) => {
                    if sender.send(Ok(chunk)).await.is_err() {
                        return;
                    }
                }
                Ok(None) => return,
                Err(error) => {
                    let error = io::Error::new(io::ErrorKind::TimedOut, error.to_string());
                    let _ = sender.send(Err(error)).await;
                    return;
                }
            }
        }
    });
    Body::from_stream(ReceiverStream::new(receiver))
}
