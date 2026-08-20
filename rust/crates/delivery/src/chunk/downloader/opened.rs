use super::ChunkSpec;
use crate::chunk::cancel::CancelToken;
use anyhow::{Context, Result};
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::response_limits::validate_response_headers;
use reqwest::header::{ACCEPT_ENCODING, IF_RANGE, RANGE};
use reqwest::Response;
use std::time::Duration;
use tokio::time::Instant;

pub(super) struct OpenedResponse {
    pub response: Response,
    pub ttfb: Duration,
}

pub(super) enum Opened {
    Response(OpenedResponse),
    Cancelled,
}

pub(super) async fn send(spec: &ChunkSpec<'_>, cancel: &CancelToken) -> Result<Opened> {
    let mut request = spec
        .client
        .get(spec.url)?
        .header(ACCEPT_ENCODING, "identity");
    if let RetrievalRequest::FetchRange { bytes, .. } = spec.request {
        let range = format!("bytes={}-{}", bytes.start, bytes.end - 1);
        request = request.header(RANGE, range);
        if let Some(generation) = spec.continuation {
            request = request.header(IF_RANGE, generation.strong_etag());
        }
    }
    let started = Instant::now();
    let response = tokio::select! {
        biased;
        _ = cancel.cancelled() => return Ok(Opened::Cancelled),
        response = tokio::time::timeout(spec.timeouts.headers, request.send()) => response,
    }
    .context("chunk response headers timed out")?
    .context("chunk request failed")?;
    validate_response_headers(response.headers())?;
    let response = response
        .error_for_status()
        .context("chunk request rejected")?;
    require_identity_encoding(response.headers())
        .context("encoded response cannot be assembled into media bytes")?;
    Ok(Opened::Response(OpenedResponse {
        response,
        ttfb: started.elapsed(),
    }))
}
