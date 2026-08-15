use anyhow::{bail, ensure, Context, Result};
use ghostr_hls_manifest::hls_manifest::MAX_HLS_MANIFEST_BYTES;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

pub(super) const MAX_HLS_ASSET_BYTES: usize = 8 * 1024 * 1024;
const HLS_HEADERS_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) struct FetchedObject {
    pub request_url: String,
    pub final_url: Url,
    pub body: Arc<[u8]>,
    pub content_type: Option<String>,
}

pub(super) async fn manifest(client: &dyn MediaHttpRequests, url: &str) -> Result<FetchedObject> {
    fetch(client, url, MAX_HLS_MANIFEST_BYTES, true).await
}

pub(super) async fn asset(client: &dyn MediaHttpRequests, url: &Url) -> Result<FetchedObject> {
    fetch(client, url.as_str(), MAX_HLS_ASSET_BYTES, false).await
}

async fn fetch(
    client: &dyn MediaHttpRequests,
    url: &str,
    limit: usize,
    require_manifest: bool,
) -> Result<FetchedObject> {
    let request = client.get(url)?.header(ACCEPT_ENCODING, "identity");
    let mut response = tokio::time::timeout(HLS_HEADERS_TIMEOUT, request.send())
        .await
        .context("HLS response headers timed out")??
        .error_for_status()
        .context("HLS object request failed")?;
    require_identity_encoding(&response)?;
    if require_manifest {
        require_manifest_type(&response)?;
    }
    let final_url = response.url().clone();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.context("read HLS object")? {
        ensure!(
            body.len().saturating_add(chunk.len()) <= limit,
            "HLS object exceeds its byte limit"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(FetchedObject {
        request_url: url.to_owned(),
        final_url,
        body: Arc::from(body),
        content_type,
    })
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
