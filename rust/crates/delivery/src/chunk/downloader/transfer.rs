use super::{opened, outcome, reply, streamed};
use super::{
    ChunkResult, ChunkSpec, HttpResponseEvidence, OriginGeneration, ResponseObservation,
    ResponseWriteMode,
};
use crate::chunk::cancel::CancelToken;
use crate::chunk::response::{classify, ResponseReply};
use crate::chunk::sink::ChunkWrite;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::RetrievalRequest;

pub async fn run<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    cancel: &CancelToken,
    network: Option<&NetworkThrottle>,
    traffic: &mut dyn ChunkTraffic,
) -> anyhow::Result<ChunkResult> {
    let opened = match opened::send(spec, cancel).await? {
        opened::Opened::Response(opened) => opened,
        opened::Opened::Cancelled => return Ok(outcome::cancelled_after_request()),
    };
    traffic.opened(opened.ttfb);
    let response = opened.response;
    let length = response.content_length();
    let reply = classify(&response, spec.request, spec.continuation.is_some())?;
    let total = reply::total(&reply, length);
    let evidence = HttpResponseEvidence::from_response(&response);
    let generation = OriginGeneration::from_response(&response, total)?;
    let input = TransferInput {
        response,
        spec,
        sink,
        cancel,
        network,
        traffic,
        generation,
        length,
        evidence,
    };
    dispatch(input, reply).await
}

struct TransferInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    response: reqwest::Response,
    spec: &'a ChunkSpec<'spec>,
    sink: &'a W,
    cancel: &'a CancelToken,
    network: Option<&'a NetworkThrottle>,
    traffic: &'a mut dyn ChunkTraffic,
    generation: OriginGeneration,
    length: Option<u64>,
    evidence: HttpResponseEvidence,
}

async fn dispatch<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    reply: ResponseReply,
) -> anyhow::Result<ChunkResult> {
    match reply {
        ResponseReply::Ignored { range_support } => ignored(input, range_support),
        ResponseReply::Partial { range, total } => partial(input, range, total).await,
        ResponseReply::Body {
            request,
            range_support,
            promoted,
        } => body(input, request, range_support, promoted).await,
    }
}

fn ignored<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    range_support: Option<bool>,
) -> anyhow::Result<ChunkResult> {
    input
        .traffic
        .response_observed(ResponseObservation::Ignored {
            total: input.length,
            range_support,
        });
    Ok(outcome::range_ignored(input.length, range_support))
}

async fn partial<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    range: ghostr_engine::ByteRange,
    total: Option<u64>,
) -> anyhow::Result<ChunkResult> {
    let returned = range_spec(input.spec, range);
    receive(ReceiveArgs {
        input,
        returned,
        range_support: Some(true),
        promoted: false,
        mode: ResponseWriteMode::Sparse,
        observation: ResponseObservation::Partial { range, total },
        total,
    })
    .await
}

async fn body<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    request: RetrievalRequest,
    range_support: Option<bool>,
    promoted: bool,
) -> anyhow::Result<ChunkResult> {
    let returned = reply::body_spec(input.spec, request);
    let mode = response_mode(request);
    if mode == ResponseWriteMode::Sparse && !input.generation.is_resumable() {
        return ignored(input, range_support);
    }
    let observation = ResponseObservation::Body {
        request,
        total: input.length,
        range_support,
        promoted,
    };
    let total = input.length;
    receive(ReceiveArgs {
        input,
        returned,
        range_support,
        promoted,
        mode,
        observation,
        total,
    })
    .await
}

struct ReceiveArgs<'a, 'spec, W: ChunkWrite + ?Sized> {
    input: TransferInput<'a, 'spec, W>,
    returned: ChunkSpec<'spec>,
    range_support: Option<bool>,
    promoted: bool,
    mode: ResponseWriteMode,
    observation: ResponseObservation,
    total: Option<u64>,
}

async fn receive<W: ChunkWrite + ?Sized>(
    args: ReceiveArgs<'_, '_, W>,
) -> anyhow::Result<ChunkResult> {
    let ReceiveArgs {
        input,
        returned,
        range_support,
        promoted,
        mode,
        observation,
        total,
    } = args;
    streamed::receive(streamed::ReceiveInput {
        response: input.response,
        spec: &returned,
        generation: &input.generation,
        sink: input.sink,
        cancel: input.cancel,
        network: input.network,
        traffic: input.traffic,
        total,
        range_support,
        range_ignored: false,
        promoted,
        mode,
        observation,
        evidence: input.evidence,
    })
    .await
}

fn range_spec<'a>(spec: &ChunkSpec<'a>, range: ghostr_engine::ByteRange) -> ChunkSpec<'a> {
    ChunkSpec {
        client: spec.client,
        url: spec.url,
        request: RetrievalRequest::FetchRange {
            bytes: range,
            promotion: None,
        },
        continuation: spec.continuation,
        timeouts: spec.timeouts,
    }
}

fn response_mode(request: RetrievalRequest) -> ResponseWriteMode {
    match request {
        RetrievalRequest::FetchWhole { contract, .. } => {
            ResponseWriteMode::SingleResponse(contract)
        }
        RetrievalRequest::FetchRange { .. } => ResponseWriteMode::Sparse,
    }
}
