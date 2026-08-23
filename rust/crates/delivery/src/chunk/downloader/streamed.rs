use super::{
    ChunkResult, ChunkSpec, HttpResponseEvidence, OpenedResponse, ResponseAdmission,
    ResponseObservation,
};
use crate::chunk::cancel::CancelToken;
use crate::chunk::generation::OriginGeneration;
use crate::chunk::sink::{ChunkWrite, LocalStoreFailure, ResponseWriteMode};
use crate::chunk::stream::{stream_into, StreamInput, Streamed};
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use anyhow::{Context, Result};
use ghostr_net::media_request_executor::MediaResponse;

pub(super) struct ReceiveInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    pub response: MediaResponse,
    pub spec: &'a ChunkSpec<'spec>,
    pub generation: &'a OriginGeneration,
    pub sink: &'a W,
    pub cancel: &'a CancelToken,
    pub network: Option<&'a NetworkThrottle>,
    pub traffic: &'a mut dyn ChunkTraffic,
    pub total: Option<u64>,
    pub range_support: Option<bool>,
    pub range_ignored: bool,
    pub promoted: bool,
    pub mode: ResponseWriteMode,
    pub observation: ResponseObservation,
    pub evidence: HttpResponseEvidence,
}

struct Completion<'a, 'spec, W: ChunkWrite + ?Sized> {
    spec: &'a ChunkSpec<'spec>,
    generation: &'a OriginGeneration,
    sink: &'a W,
    total: Option<u64>,
    range_support: Option<bool>,
    range_ignored: bool,
    promoted: bool,
    mode: ResponseWriteMode,
}

pub(super) async fn receive<W: ChunkWrite + ?Sized>(
    mut input: ReceiveInput<'_, '_, W>,
) -> Result<ChunkResult> {
    let opened = input.opened_response();
    input.traffic.response_observed(opened.clone());
    if !authorize(&mut input, opened).await? {
        return Ok(rejected(&input.completion()));
    }
    input
        .sink
        .accept(input.generation, input.mode)
        .await
        .context(LocalStoreFailure)?;
    let completion = input.completion();
    let streamed = stream(input).await;
    match streamed {
        Ok(streamed) => finish(&completion, streamed).await,
        Err(error) => abort(&completion, error).await,
    }
}

async fn authorize<W: ChunkWrite + ?Sized>(
    input: &mut ReceiveInput<'_, '_, W>,
    opened: OpenedResponse,
) -> Result<bool> {
    let admission = tokio::select! {
        biased;
        _ = input.cancel.cancelled() => ResponseAdmission::Reject,
        result = input.traffic.authorize_response(opened) => result?,
    };
    Ok(admission == ResponseAdmission::Proceed)
}

async fn stream<W: ChunkWrite + ?Sized>(input: ReceiveInput<'_, '_, W>) -> Result<Streamed> {
    stream_into(StreamInput {
        response: input.response,
        spec: input.spec,
        generation: input.generation,
        sink: input.sink,
        mode: input.mode,
        cancel: input.cancel,
        network: input.network,
        traffic: input.traffic,
        response_evidence: input.evidence.clone(),
    })
    .await
}

impl<'a, 'spec, W: ChunkWrite + ?Sized> ReceiveInput<'a, 'spec, W> {
    fn opened_response(&self) -> OpenedResponse {
        OpenedResponse::new(
            self.observation,
            self.generation.resumable(),
            self.mode,
            self.evidence.clone(),
        )
    }

    fn completion(&self) -> Completion<'a, 'spec, W> {
        Completion {
            spec: self.spec,
            generation: self.generation,
            sink: self.sink,
            total: self.total,
            range_support: self.range_support,
            range_ignored: self.range_ignored,
            promoted: self.promoted,
            mode: self.mode,
        }
    }
}

fn rejected<W: ChunkWrite + ?Sized>(input: &Completion<'_, '_, W>) -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        range_support: input.range_support,
        range_ignored: input.range_ignored,
        cancelled: true,
        total_bytes: input.total,
        promoted: input.promoted,
        request_started: true,
    }
}

async fn finish<W: ChunkWrite + ?Sized>(
    input: &Completion<'_, '_, W>,
    streamed: Streamed,
) -> Result<ChunkResult> {
    let total = streamed
        .whole_body_completion
        .map(|completion| completion.total_bytes())
        .or(input.total);
    let range = input.spec.request.requested_bytes();
    let complete = !streamed.cancelled
        && total.is_some_and(|total| range.start == 0 && streamed.bytes == total);
    let kept = input
        .sink
        .finish(input.generation, input.mode, total, complete)
        .await
        .context(LocalStoreFailure)?;
    Ok(ChunkResult {
        bytes_written: if kept { streamed.bytes } else { 0 },
        range_support: input.range_support,
        range_ignored: input.range_ignored,
        cancelled: streamed.cancelled || !kept,
        total_bytes: total,
        promoted: input.promoted,
        request_started: true,
    })
}

async fn abort<W: ChunkWrite + ?Sized>(
    input: &Completion<'_, '_, W>,
    error: anyhow::Error,
) -> Result<ChunkResult> {
    if let Err(rollback) = input
        .sink
        .finish(input.generation, input.mode, input.total, false)
        .await
    {
        return Err(rollback
            .context(format!("rollback after {error:#}"))
            .context(LocalStoreFailure));
    }
    Err(error)
}
