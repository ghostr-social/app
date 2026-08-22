use super::deadline;
use super::telemetry::{http_reason, FetchProblem};
use super::{require_manifest_type, FetchRuntime, FetchSpec};
use anyhow::Context;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::MediaResponse;
use ghostr_net::response_limits::validate_response_headers;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING};
use reqwest::StatusCode;
use tokio::time::Instant;

pub(super) struct OpenedObject {
    pub response: MediaResponse,
}

pub(super) async fn open(
    runtime: FetchRuntime<'_>,
    spec: FetchSpec<'_>,
) -> Result<OpenedObject, FetchProblem> {
    let request = runtime
        .requests
        .get(spec.url, spec.priority)
        .map_err(|error| FetchProblem::new(error, ErrorReason::Policy))?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
    let admission_deadline = spec
        .admission_fence
        .unwrap_or(runtime.deadline)
        .min(runtime.deadline);
    let commitment_limited = spec
        .admission_fence
        .is_some_and(|fence| fence <= runtime.deadline);
    let admitted = tokio::time::timeout_at(admission_deadline, request.admit())
        .await
        .map_err(|error| admission_timeout(error, commitment_limited))?
        .map_err(|error| FetchProblem::new(error, ErrorReason::Policy))?;
    if spec
        .admission_fence
        .is_some_and(|fence| Instant::now() >= fence)
    {
        return Err(ownership_expired());
    }
    runtime
        .progress
        .mark_admitted(runtime.requests, spec.url, runtime.network);
    let header_deadline = runtime.deadline.min(Instant::now() + spec.timeouts.headers);
    let timeout_context = deadline::header_context(header_deadline, runtime.deadline);
    let sending = admitted.send_with_redirect_deadline(header_deadline);
    let response = tokio::time::timeout_at(header_deadline, sending)
        .await
        .map_err(|error| timeout(error, timeout_context))?
        .map_err(FetchProblem::transport)?;
    runtime.progress.received(runtime.requests, &response);
    Ok(OpenedObject {
        response: validate_open_response(response, spec.require_manifest, spec.limit)?,
    })
}

fn validate_open_response(
    response: MediaResponse,
    require_manifest: bool,
    limit: usize,
) -> Result<MediaResponse, FetchProblem> {
    validate_response_headers(response.headers())
        .map_err(|error| FetchProblem::new(error, ErrorReason::InvalidResponse))?;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let error = anyhow::anyhow!("HLS object request failed with HTTP status {status}");
        return Err(FetchProblem::new(error, http_reason(status)));
    }
    if status != StatusCode::OK {
        let error = anyhow::anyhow!("full HLS object response is not 200: {status}");
        return Err(FetchProblem::new(error, ErrorReason::InvalidResponse));
    }
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        let error = anyhow::anyhow!("HLS object declared length exceeds its byte limit");
        return Err(FetchProblem::new(error, ErrorReason::InvalidResponse));
    }
    require_identity_encoding(response.headers())
        .context("encoded HLS object is not cacheable")
        .map_err(|error| FetchProblem::new(error, ErrorReason::InvalidResponse))?;
    if require_manifest {
        require_manifest_type(&response)
            .map_err(|error| FetchProblem::new(error, ErrorReason::InvalidResponse))?;
    }
    Ok(response)
}

fn timeout(error: tokio::time::error::Elapsed, context: &'static str) -> FetchProblem {
    FetchProblem::new(
        anyhow::Error::new(error).context(context),
        ErrorReason::Timeout,
    )
}

fn admission_timeout(error: tokio::time::error::Elapsed, commitment_limited: bool) -> FetchProblem {
    match commitment_limited {
        true => ownership_expired(),
        false => timeout(error, "HLS object transfer timed out"),
    }
}

fn ownership_expired() -> FetchProblem {
    FetchProblem::new(
        anyhow::anyhow!("HLS WARP commitment expired before request admission"),
        ErrorReason::Policy,
    )
}
