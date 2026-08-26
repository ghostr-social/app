use super::ChunkSpec;
use crate::chunk::cancel::CancelToken;
use crate::chunk::traffic::ChunkTraffic;
use anyhow::{Context as _, Result};
use core::future::{poll_fn, Future as _};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{AdmittedMediaRequest, MediaRequest, MediaResponse};
use ghostr_net::response_limits::validate_response_headers;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING, IF_RANGE, RANGE};
use tokio::time::Instant;

#[cfg(test)]
#[path = "opened/network_admission_test.rs"]
mod network_admission_test;
#[cfg(test)]
#[path = "opened/post_admission_cancellation_test.rs"]
mod post_admission_cancellation_test;

pub(super) struct OpenedResponse {
    pub response: MediaResponse,
    pub observed: ghostr_engine::evidence::EvidenceTime,
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
        () = cancel.cancelled() => Ok(None),
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
    if cancel.is_cancelled() {
        return Ok(Opened::CancelledBeforeRequest);
    }
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
        response = tokio::time::timeout_at(deadline, tracked) => response,
        () = cancel.cancelled() => {
            return Ok(cancelled(request_started.load(Ordering::Relaxed)));
        },
    }
    .context("chunk response headers timed out")?
    .context("chunk request failed")?;
    let observed = crate::manager::time::evidence_time();
    let ttfb = response.origin_elapsed(started.elapsed());
    traffic.opened(ttfb);
    validate_response_headers(response.headers())
        .context(super::ResponseFailure::InvalidResponse)?;
    let response = accept_status(response, observed, traffic)?;
    accept_encoding(&response, observed, traffic)?;
    Ok(Opened::Response(OpenedResponse { response, observed }))
}

fn accept_status(
    response: MediaResponse,
    observed: ghostr_engine::evidence::EvidenceTime,
    traffic: &mut dyn ChunkTraffic,
) -> Result<MediaResponse> {
    if response.status().is_success() {
        return Ok(response);
    }
    observe_rejection(
        &response,
        observed,
        super::ResponseRejection::Status,
        traffic,
    );
    response
        .error_for_status()
        .context("chunk request rejected")
}

fn accept_encoding(
    response: &MediaResponse,
    observed: ghostr_engine::evidence::EvidenceTime,
    traffic: &mut dyn ChunkTraffic,
) -> Result<()> {
    if let Err(error) = require_identity_encoding(response.headers()) {
        observe_rejection(
            response,
            observed,
            super::ResponseRejection::ContentEncoding,
            traffic,
        );
        return Err(error)
            .context(super::ResponseFailure::InvalidResponse)
            .context("encoded response cannot be assembled into media bytes");
    }
    Ok(())
}

fn observe_rejection(
    response: &MediaResponse,
    observed: ghostr_engine::evidence::EvidenceTime,
    rejection: super::ResponseRejection,
    traffic: &mut dyn ChunkTraffic,
) {
    let evidence = super::HttpResponseEvidence::from_response(response, observed).provenance_only();
    traffic.response_observed(super::OpenedResponse::new(
        super::ResponseObservation::Rejected(rejection),
        None,
        crate::chunk::sink::ResponseWriteMode::Sparse,
        evidence,
    ));
}

fn cancelled(request_started: bool) -> Opened {
    if request_started {
        Opened::CancelledAfterRequest
    } else {
        Opened::CancelledBeforeRequest
    }
}
