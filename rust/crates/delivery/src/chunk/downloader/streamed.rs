use super::{
    ChunkResult, ChunkSpec, HttpResponseEvidence, OpenedResponse, ResponseAdmission,
    ResponseObservation,
};
use crate::chunk::cancel::CancelToken;
use crate::chunk::generation::OriginGeneration;
use crate::chunk::sink::ChunkWrite;
use crate::chunk::sink::ResponseWriteMode;
use crate::chunk::stream::{stream_into, StreamInput, Streamed};
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use anyhow::Result;
use reqwest::Response;

pub(super) struct ReceiveInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    pub response: Response,
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
    input: ReceiveInput<'_, '_, W>,
) -> Result<ChunkResult> {
    let ReceiveInput {
        response,
        spec,
        generation,
        sink,
        mode,
        cancel,
        network,
        traffic,
        total,
        range_support,
        range_ignored,
        promoted,
        observation,
        evidence,
    } = input;
    let opened = OpenedResponse::new(observation, generation.resumable(), mode, evidence);
    let admission = tokio::select! {
        biased;
        _ = cancel.cancelled() => ResponseAdmission::Reject,
        result = traffic.authorize_response(opened) => result?,
    };
    if admission == ResponseAdmission::Reject {
        return Ok(rejected(&Completion {
            spec,
            generation,
            sink,
            total,
            range_support,
            range_ignored,
            promoted,
            mode,
        }));
    }
    sink.accept(generation, mode).await?;
    let streamed = stream_into(StreamInput {
        response,
        spec,
        generation,
        sink,
        mode,
        cancel,
        network,
        traffic,
    })
    .await;
    let completion = Completion {
        spec,
        generation,
        sink,
        total,
        range_support,
        range_ignored,
        promoted,
        mode,
    };
    match streamed {
        Ok(streamed) => finish(&completion, streamed).await,
        Err(error) => abort(&completion, error).await,
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
    let total = streamed.discovered_total.or(input.total);
    let range = input.spec.request.requested_bytes();
    let complete = !streamed.cancelled
        && total.is_some_and(|total| range.start == 0 && streamed.bytes == total);
    let kept = input
        .sink
        .finish(input.generation, input.mode, total, complete)
        .await?;
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
    let _ = input
        .sink
        .finish(input.generation, input.mode, input.total, false)
        .await;
    Err(error)
}
