use super::deadline;
use super::response::{self, ResponseExtent};
use super::telemetry::FetchProblem;
use super::{FetchRuntime, FetchSpec};
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::media_request_executor::{MediaRequest, MediaResponse};
use reqwest::header::{HeaderValue, ACCEPT_ENCODING, RANGE};
use tokio::time::Instant;

pub(super) struct OpenedObject {
    pub response: MediaResponse,
    pub extent: ResponseExtent,
}

pub(super) async fn open(
    runtime: FetchRuntime<'_>,
    spec: FetchSpec<'_>,
) -> Result<OpenedObject, FetchProblem> {
    let request = request(runtime, spec)?;
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
        .map_err(|error| FetchProblem::neutral(error, ErrorReason::Policy))?;
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
    let extent = response::validate(&response, spec)?;
    Ok(OpenedObject { response, extent })
}

fn request(runtime: FetchRuntime<'_>, spec: FetchSpec<'_>) -> Result<MediaRequest, FetchProblem> {
    let end = spec.request_end();
    let range = format!("bytes={}-{}", spec.object.offset, end.saturating_sub(1));
    let request = runtime
        .requests
        .get(spec.url, spec.priority)
        .map_err(|error| FetchProblem::new(error, ErrorReason::Policy))?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"))
        .header(RANGE, range.parse().map_err(policy)?);
    Ok(request)
}

fn policy(error: reqwest::header::InvalidHeaderValue) -> FetchProblem {
    FetchProblem::new(anyhow::Error::new(error), ErrorReason::Policy)
}

fn timeout(error: tokio::time::error::Elapsed, context: &'static str) -> FetchProblem {
    FetchProblem::new(
        anyhow::Error::new(error).context(context),
        ErrorReason::Timeout,
    )
}

fn admission_timeout(error: tokio::time::error::Elapsed, commitment_limited: bool) -> FetchProblem {
    if commitment_limited {
        ownership_expired()
    } else {
        FetchProblem::neutral(
            anyhow::Error::new(error).context("HLS request admission timed out"),
            ErrorReason::Timeout,
        )
    }
}

fn ownership_expired() -> FetchProblem {
    FetchProblem::neutral(
        anyhow::anyhow!("HLS WARP commitment expired before request admission"),
        ErrorReason::Policy,
    )
}
