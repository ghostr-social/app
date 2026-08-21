use anyhow::{bail, ensure, Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_hls_manifest::hls_manifest::{MAX_HLS_ASSET_BYTES, MAX_HLS_MANIFEST_BYTES};
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_TYPE};
use reqwest::StatusCode;
use std::sync::Arc;
use url::Url;

mod deadline;
#[cfg(test)]
mod tests;

pub(super) struct FetchedObject {
    pub request_url: String,
    pub final_url: Url,
    pub body: Arc<[u8]>,
    pub content_type: Option<String>,
}

#[derive(Clone, Copy)]
struct FetchSpec<'a> {
    url: &'a str,
    limit: usize,
    require_manifest: bool,
    timeouts: HlsTransferTimeouts,
    priority: PreemptionAuthority,
}

pub(super) async fn manifest(
    requests: &MediaRequestExecutor,
    url: &str,
    priority: PreemptionAuthority,
) -> Result<FetchedObject> {
    fetch(
        requests,
        FetchSpec {
            url,
            limit: MAX_HLS_MANIFEST_BYTES,
            require_manifest: true,
            timeouts: HlsTransferTimeouts::default(),
            priority,
        },
    )
    .await
}

pub(super) async fn asset(
    requests: &MediaRequestExecutor,
    url: &Url,
    priority: PreemptionAuthority,
) -> Result<FetchedObject> {
    asset_with_timeouts(
        requests,
        url.as_str(),
        HlsTransferTimeouts::default(),
        priority,
    )
    .await
}

async fn asset_with_timeouts(
    requests: &MediaRequestExecutor,
    url: &str,
    timeouts: HlsTransferTimeouts,
    priority: PreemptionAuthority,
) -> Result<FetchedObject> {
    fetch(
        requests,
        FetchSpec {
            url,
            limit: MAX_HLS_ASSET_BYTES,
            require_manifest: false,
            timeouts,
            priority,
        },
    )
    .await
}

async fn fetch(requests: &MediaRequestExecutor, spec: FetchSpec<'_>) -> Result<FetchedObject> {
    let deadline = tokio::time::Instant::now() + spec.timeouts.total;
    tokio::time::timeout_at(deadline, fetch_before_total(requests, spec, deadline))
        .await
        .context("HLS object transfer timed out")?
}

async fn fetch_before_total(
    requests: &MediaRequestExecutor,
    spec: FetchSpec<'_>,
    deadline: tokio::time::Instant,
) -> Result<FetchedObject> {
    let mut response = open(requests, spec, deadline).await?;
    let final_url = response.url().clone();
    let content_type = content_type(&response);
    let body = read_body(&mut response, spec.limit, spec.timeouts.idle).await?;
    Ok(FetchedObject {
        request_url: spec.url.to_owned(),
        final_url,
        body: Arc::from(body),
        content_type,
    })
}

async fn open(
    requests: &MediaRequestExecutor,
    spec: FetchSpec<'_>,
    deadline: tokio::time::Instant,
) -> Result<MediaResponse> {
    let request = requests
        .get(spec.url, spec.priority)?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
    let admitted = tokio::time::timeout_at(deadline, request.admit())
        .await
        .context("HLS object transfer timed out")??;
    let header_deadline = deadline.min(tokio::time::Instant::now() + spec.timeouts.headers);
    let timeout_context = deadline::header_context(header_deadline, deadline);
    let response = tokio::time::timeout_at(header_deadline, admitted.send())
        .await
        .context(timeout_context)??;
    validate_open_response(response, spec.require_manifest)
}

fn validate_open_response(
    response: MediaResponse,
    require_manifest: bool,
) -> Result<MediaResponse> {
    validate_response_headers(response.headers())?;
    let response = response
        .error_for_status()
        .context("HLS object request failed")?;
    ensure!(
        response.status() == StatusCode::OK,
        "full HLS object response is not 200"
    );
    require_identity_encoding(response.headers()).context("encoded HLS object is not cacheable")?;
    if require_manifest {
        require_manifest_type(&response)?;
    }
    Ok(response)
}

async fn read_body(
    response: &mut MediaResponse,
    limit: usize,
    idle: std::time::Duration,
) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) = next_chunk(response, idle).await? {
        ensure!(
            body.len().saturating_add(chunk.len()) <= limit,
            "HLS object exceeds its byte limit"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn next_chunk(
    response: &mut MediaResponse,
    idle: std::time::Duration,
) -> Result<Option<bytes::Bytes>> {
    tokio::time::timeout(idle, response.chunk())
        .await
        .context("HLS object body idle timed out")?
        .context("read HLS object")
}

fn content_type(response: &MediaResponse) -> Option<String> {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn require_manifest_type(response: &MediaResponse) -> Result<()> {
    let media_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if matches!(
        media_type,
        Some(
            "application/vnd.apple.mpegurl"
                | "application/x-mpegurl"
                | "audio/mpegurl"
                | "audio/x-mpegurl"
        )
    ) {
        return Ok(());
    }
    bail!("HLS manifest has an unsupported content type")
}
