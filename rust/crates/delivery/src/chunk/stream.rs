//! Streams one origin response into its granted store transaction.

use crate::chunk::cancel::CancelToken;
use crate::chunk::downloader::{ChunkSpec, HttpResponseEvidence};
use crate::chunk::generation::OriginGeneration;
use crate::chunk::sink::{ChunkWrite, ResponseWriteMode};
use crate::chunk::traffic::{ChunkTraffic, WholeBodyCompletion};
use crate::debug::network::NetworkThrottle;
use anyhow::{bail, ensure, Context as _, Result};
use core::future::Future;
use core::num::NonZeroU64;
use core::time::Duration;
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract};
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaResponse;

mod progress;
pub(crate) use progress::Streamed;
mod store;
use store::{store_bytes, StoreInput};

#[cfg(test)]
#[path = "stream/cancellation_priority_test.rs"]
mod cancellation_priority_test;

pub(crate) struct StreamInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    pub response: MediaResponse,
    pub spec: &'a ChunkSpec<'spec>,
    pub generation: &'a OriginGeneration,
    pub sink: &'a W,
    pub mode: ResponseWriteMode,
    pub cancel: &'a CancelToken,
    pub network: Option<&'a NetworkThrottle>,
    pub traffic: &'a mut dyn ChunkTraffic,
    pub response_evidence: HttpResponseEvidence,
}

pub(crate) async fn stream_into<W: ChunkWrite + ?Sized>(
    mut input: StreamInput<'_, '_, W>,
) -> Result<Streamed> {
    match input.spec.request {
        RetrievalRequest::FetchRange { bytes, .. } => stream_range(&mut input, bytes).await,
        RetrievalRequest::FetchWhole { contract, .. } => stream_whole(&mut input, contract).await,
    }
}

async fn stream_range<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
    range: ByteRange,
) -> Result<Streamed> {
    let mut written = 0;
    while written < range.len() {
        let Some(chunk) = next_input(input).await? else {
            return ended_range(written, input.cancel);
        };
        let take = chunk.len().min((range.len() - written) as usize);
        let stored = store_bytes(
            StoreInput::from(&*input),
            range.start + written,
            &chunk[..take],
        )
        .await?;
        written += stored.bytes;
        if stored.cancelled {
            return Ok(stopped(written));
        }
    }
    Ok(completed_range(written))
}

async fn stream_whole<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
    contract: WholeBodyContract,
) -> Result<Streamed> {
    let mut written = 0_u64;
    loop {
        let Some(chunk) = next_input(input).await? else {
            let streamed = ended_whole(written, contract, input)?;
            if let Some(completion) = streamed.whole_body_completion.clone() {
                input.traffic.whole_body_completed(completion);
            }
            return Ok(streamed);
        };
        crate::chunk::whole_body_limit::WholeBodyLimitReached::check(
            written,
            chunk.len() as u64,
            contract,
        )?;
        let stored = store_bytes(StoreInput::from(&*input), written, &chunk).await?;
        written += stored.bytes;
        if stored.cancelled {
            return Ok(stopped(written));
        }
    }
}

async fn next_input<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
) -> Result<Option<bytes::Bytes>> {
    let body = next_chunk(&mut input.response, input.spec.timeouts.idle);
    let next = select_next(input.cancel, body).await?;
    if let Some(chunk) = &next {
        input.traffic.received(chunk.len() as u64);
    }
    Ok(next)
}

async fn select_next<F>(cancel: &CancelToken, body: F) -> Result<Option<bytes::Bytes>>
where
    F: Future<Output = Result<Option<bytes::Bytes>>>,
{
    if cancel.is_cancelled() {
        return Ok(None);
    }
    tokio::select! {
        biased;
        () = cancel.cancelled() => Ok(None),
        chunk = body => chunk,
    }
}

async fn next_chunk(response: &mut MediaResponse, idle: Duration) -> Result<Option<bytes::Bytes>> {
    tokio::time::timeout(idle, response.chunk())
        .await
        .context("chunk body read timed out")?
        .context("chunk body read failed")
}

fn ended_range(written: u64, cancel: &CancelToken) -> Result<Streamed> {
    if cancel.is_cancelled() {
        return Ok(stopped(written));
    }
    bail!("response body ended before its advertised range")
}

fn ended_whole(
    written: u64,
    contract: WholeBodyContract,
    input: &StreamInput<'_, '_, impl ChunkWrite + ?Sized>,
) -> Result<Streamed> {
    if input.cancel.is_cancelled() {
        return Ok(stopped(written));
    }
    ensure!(written > 0, "whole response body is empty");
    if let WholeBodyContract::Exact { expected_bytes } = contract {
        ensure!(written == expected_bytes, "whole response length changed");
    }
    Ok(Streamed {
        bytes: written,
        cancelled: false,
        whole_body_completion: NonZeroU64::new(written)
            .map(|total| WholeBodyCompletion::at_network_eof(total, &input.response_evidence)),
    })
}

fn stopped(bytes: u64) -> Streamed {
    Streamed {
        bytes,
        cancelled: true,
        whole_body_completion: None,
    }
}

fn completed_range(bytes: u64) -> Streamed {
    Streamed {
        bytes,
        cancelled: false,
        whole_body_completion: None,
    }
}
