use super::ChunkSpec;
use anyhow::{Context, Result};
use reqwest::header::RANGE;
use reqwest::Response;
use std::time::Duration;
use tokio::time::Instant;

pub(super) struct OpenedResponse {
    pub response: Response,
    pub ttfb: Duration,
}

pub(super) async fn send_ranged(spec: &ChunkSpec<'_>) -> Result<OpenedResponse> {
    let header = format!("bytes={}-{}", spec.range.start, spec.range.end - 1);
    let request = spec.client.get(spec.url)?.header(RANGE, header);
    let started = Instant::now();
    let response = tokio::time::timeout(spec.timeouts.headers, request.send())
        .await
        .context("chunk response headers timed out")?
        .context("chunk request failed")?
        .error_for_status()
        .context("chunk request rejected")?;
    Ok(OpenedResponse {
        response,
        ttfb: started.elapsed(),
    })
}
