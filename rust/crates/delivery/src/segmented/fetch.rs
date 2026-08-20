use anyhow::{bail, ensure, Context, Result};
use ghostr_hls_manifest::hls_manifest::MAX_HLS_MANIFEST_BYTES;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use std::sync::Arc;
use url::Url;

#[cfg(test)]
mod tests;

pub(super) const MAX_HLS_ASSET_BYTES: usize = 8 * 1024 * 1024;

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
}

pub(super) async fn manifest(client: &dyn MediaHttpRequests, url: &str) -> Result<FetchedObject> {
    fetch(
        client,
        FetchSpec {
            url,
            limit: MAX_HLS_MANIFEST_BYTES,
            require_manifest: true,
            timeouts: HlsTransferTimeouts::default(),
        },
    )
    .await
}

pub(super) async fn asset(client: &dyn MediaHttpRequests, url: &Url) -> Result<FetchedObject> {
    asset_with_timeouts(client, url.as_str(), HlsTransferTimeouts::default()).await
}

async fn asset_with_timeouts(
    client: &dyn MediaHttpRequests,
    url: &str,
    timeouts: HlsTransferTimeouts,
) -> Result<FetchedObject> {
    fetch(
        client,
        FetchSpec {
            url,
            limit: MAX_HLS_ASSET_BYTES,
            require_manifest: false,
            timeouts,
        },
    )
    .await
}

async fn fetch(client: &dyn MediaHttpRequests, spec: FetchSpec<'_>) -> Result<FetchedObject> {
    tokio::time::timeout(spec.timeouts.total, fetch_before_total(client, spec))
        .await
        .context("HLS object transfer timed out")?
}

async fn fetch_before_total(
    client: &dyn MediaHttpRequests,
    spec: FetchSpec<'_>,
) -> Result<FetchedObject> {
    let mut response = open(client, spec).await?;
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

async fn open(client: &dyn MediaHttpRequests, spec: FetchSpec<'_>) -> Result<reqwest::Response> {
    let request = client.get(spec.url)?.header(ACCEPT_ENCODING, "identity");
    let response = tokio::time::timeout(spec.timeouts.headers, request.send())
        .await
        .context("HLS response headers timed out")??;
    validate_response_headers(response.headers())?;
    let response = response
        .error_for_status()
        .context("HLS object request failed")?;
    require_identity_encoding(&response)?;
    if spec.require_manifest {
        require_manifest_type(&response)?;
    }
    Ok(response)
}

async fn read_body(
    response: &mut reqwest::Response,
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
    response: &mut reqwest::Response,
    idle: std::time::Duration,
) -> Result<Option<bytes::Bytes>> {
    tokio::time::timeout(idle, response.chunk())
        .await
        .context("HLS object body idle timed out")?
        .context("read HLS object")
}

fn content_type(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn require_identity_encoding(response: &reqwest::Response) -> Result<()> {
    let Some(value) = response.headers().get(CONTENT_ENCODING) else {
        return Ok(());
    };
    if value
        .to_str()
        .ok()
        .is_some_and(|value| value.eq_ignore_ascii_case("identity"))
    {
        return Ok(());
    }
    bail!("encoded HLS object is not cacheable")
}

fn require_manifest_type(response: &reqwest::Response) -> Result<()> {
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
