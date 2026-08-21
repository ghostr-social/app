use super::ChunkSpec;
use crate::chunk::cancel::CancelToken;
use crate::chunk::traffic::ChunkTraffic;
use anyhow::{Context, Result};
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{AdmittedMediaRequest, MediaRequest, MediaResponse};
use ghostr_net::response_limits::validate_response_headers;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING, IF_RANGE, RANGE};
use std::future::{poll_fn, Future};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::Instant;

#[cfg(test)]
#[path = "opened/post_admission_cancellation_test.rs"]
mod post_admission_cancellation_test;

pub(super) struct OpenedResponse {
    pub response: MediaResponse,
    pub ttfb: Duration,
}

pub(super) enum Opened {
    Response(OpenedResponse),
    CancelledBeforeRequest,
    CancelledAfterRequest,
}

pub(super) async fn send(
    spec: &ChunkSpec<'_>,
    cancel: &CancelToken,
    traffic: &mut dyn ChunkTraffic,
) -> Result<Opened> {
    let request = request(spec)?;
    let Some(admitted) = admit(request, cancel, spec.timeouts.admission).await? else {
        return Ok(Opened::CancelledBeforeRequest);
    };
    record_concurrency(spec, traffic);
    open(admitted, cancel, spec.timeouts.headers, traffic).await
}

fn request(spec: &ChunkSpec<'_>) -> Result<MediaRequest> {
    let mut request = spec
        .requests
        .get(spec.url, spec.priority)?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
    if let RetrievalRequest::FetchRange { bytes, .. } = spec.request {
        let range = format!("bytes={}-{}", bytes.start, bytes.end - 1);
        request = request.header(RANGE, range.parse()?);
        if let Some(generation) = spec.continuation {
            request = request.header(IF_RANGE, generation.strong_etag().parse()?);
        }
    }
    Ok(request)
}

async fn admit(
    request: MediaRequest,
    cancel: &CancelToken,
    wait: Duration,
) -> Result<Option<AdmittedMediaRequest>> {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Ok(None),
        admitted = request.admit_for(wait) => admitted.map(Some),
    }
}

fn record_concurrency(spec: &ChunkSpec<'_>, traffic: &mut dyn ChunkTraffic) {
    let concurrency = ghostr_engine::RequestAuthority::from_url(spec.url)
        .map(|authority| spec.requests.active_for(&authority))
        .unwrap_or(1);
    traffic.concurrency(concurrency);
}

async fn open(
    admitted: AdmittedMediaRequest,
    cancel: &CancelToken,
    headers: Duration,
    traffic: &mut dyn ChunkTraffic,
) -> Result<Opened> {
    let started = Instant::now();
    let deadline = started + headers;
    let request_started = AtomicBool::new(false);
    let sending = admitted.send_with_redirect_deadline(deadline);
    tokio::pin!(sending);
    let tracked = poll_fn(|context| {
        if !request_started.swap(true, Ordering::Relaxed) {
            traffic.request_started();
        }
        sending.as_mut().poll(context)
    });
    let response = tokio::select! {
        biased;
        _ = cancel.cancelled() => {
            return Ok(cancelled(request_started.load(Ordering::Relaxed)));
        },
        response = tokio::time::timeout_at(deadline, tracked) => response,
    }
    .context("chunk response headers timed out")?
    .context("chunk request failed")?;
    validate_response_headers(response.headers())?;
    let response = response
        .error_for_status()
        .context("chunk request rejected")?;
    require_identity_encoding(response.headers())
        .context("encoded response cannot be assembled into media bytes")?;
    let ttfb = response.origin_elapsed(started.elapsed());
    Ok(Opened::Response(OpenedResponse { response, ttfb }))
}

fn cancelled(request_started: bool) -> Opened {
    match request_started {
        true => Opened::CancelledAfterRequest,
        false => Opened::CancelledBeforeRequest,
    }
}
