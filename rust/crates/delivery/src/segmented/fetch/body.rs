use super::open::open;
use super::telemetry::{FetchProblem, FetchProgress};
use super::{content_type, FetchRuntime, FetchSpec, FetchedObject};
use anyhow::Context;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::media_request_executor::MediaResponse;
use std::mem::MaybeUninit;
use std::sync::Arc;

pub(super) async fn fetch_before_total(
    runtime: FetchRuntime<'_>,
    spec: FetchSpec<'_>,
) -> Result<FetchedObject, FetchProblem> {
    let mut opened = open(runtime, spec).await?;
    let final_url = opened.response.url().clone();
    let content_type = content_type(&opened.response);
    let cache = crate::segmented::cache::HlsCacheMetadata::from_response(
        spec.url,
        &final_url,
        opened.response.headers(),
        spec.timeouts.headers,
    );
    let expected = opened.extent.expected_bytes();
    let body = read_body(
        &mut opened.response,
        expected,
        spec.timeouts.idle,
        runtime.progress,
    )
    .await?;
    let telemetry = runtime
        .progress
        .origin()
        .expect("admitted HLS request clock");
    let offset = opened.extent.offset();
    let continuation = opened.extent.continuation(&final_url);
    Ok(FetchedObject {
        request_url: spec.url.to_owned(),
        final_url,
        body,
        content_type,
        cache,
        telemetry,
        offset,
        continuation,
    })
}

async fn read_body(
    response: &mut MediaResponse,
    limit: usize,
    idle: std::time::Duration,
    progress: &FetchProgress,
) -> Result<Arc<[u8]>, FetchProblem> {
    let mut body = Arc::<[u8]>::new_uninit_slice(limit);
    let mut written = 0usize;
    while let Some(chunk) = next_chunk(response, idle).await? {
        progress.add_network_bytes(chunk.len() as u64);
        let Some(end) = written.checked_add(chunk.len()).filter(|end| *end <= limit) else {
            return Err(FetchProblem::new(
                anyhow::anyhow!("HLS object exceeds its byte limit"),
                ErrorReason::InvalidResponse,
            ));
        };
        write_chunk(&mut body, written, end, &chunk);
        written = end;
    }
    if written != limit {
        return Err(FetchProblem::new(
            anyhow::anyhow!("HLS object body length changed"),
            ErrorReason::RangeNoncompliant,
        ));
    }
    // Every slot was initialized exactly once by `write_chunk`.
    Ok(unsafe { body.assume_init() })
}

fn write_chunk(body: &mut Arc<[MaybeUninit<u8>]>, start: usize, end: usize, chunk: &[u8]) {
    let output = Arc::get_mut(body).expect("HLS body allocation is uniquely owned");
    for (slot, byte) in output[start..end].iter_mut().zip(chunk) {
        slot.write(*byte);
    }
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
