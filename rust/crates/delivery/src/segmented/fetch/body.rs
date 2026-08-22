use super::open::open;
use super::telemetry::{FetchProblem, FetchProgress};
use super::{content_type, FetchRuntime, FetchSpec, FetchedObject};
use anyhow::Context;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::media_request_executor::MediaResponse;
use std::sync::Arc;

pub(super) async fn fetch_before_total(
    runtime: FetchRuntime<'_>,
    spec: FetchSpec<'_>,
) -> Result<FetchedObject, FetchProblem> {
    let mut opened = open(runtime, spec).await?;
    let final_url = opened.response.url().clone();
    let content_type = content_type(&opened.response);
    let body = read_body(
        &mut opened.response,
        spec.limit,
        spec.timeouts.idle,
        runtime.progress,
    )
    .await?;
    let telemetry = runtime
        .progress
        .origin()
        .expect("admitted HLS request clock");
    Ok(FetchedObject {
        request_url: spec.url.to_owned(),
        final_url,
        body: Arc::from(body),
        content_type,
        telemetry,
    })
}

async fn read_body(
    response: &mut MediaResponse,
    limit: usize,
    idle: std::time::Duration,
    progress: &FetchProgress,
) -> Result<Vec<u8>, FetchProblem> {
    let mut body = Vec::new();
    while let Some(chunk) = next_chunk(response, idle).await? {
        progress.add_network_bytes(chunk.len() as u64);
        if body.len().saturating_add(chunk.len()) > limit {
            return Err(FetchProblem::new(
                anyhow::anyhow!("HLS object exceeds its byte limit"),
                ErrorReason::InvalidResponse,
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn next_chunk(
    response: &mut MediaResponse,
    idle: std::time::Duration,
) -> Result<Option<bytes::Bytes>, FetchProblem> {
    let result = tokio::time::timeout(idle, response.chunk())
        .await
        .map_err(|error| {
            FetchProblem::new(
                anyhow::Error::new(error).context("HLS object body idle timed out"),
                ErrorReason::Timeout,
            )
        })?;
    result
        .context("read HLS object")
        .map_err(FetchProblem::transport)
}
