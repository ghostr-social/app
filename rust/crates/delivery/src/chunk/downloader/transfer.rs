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

#[cfg(test)]
#[path = "transfer/queued_cancellation_test.rs"]
mod queued_cancellation_test;

pub(super) struct TransferExecution<'a, W: ChunkWrite + ?Sized> {
    pub sink: &'a W,
    pub cancel: &'a CancelToken,
    pub network: Option<&'a NetworkThrottle>,
    pub traffic: &'a mut dyn ChunkTraffic,
}

pub async fn run<'a, 'spec, W: ChunkWrite + ?Sized>(
    spec: &'a ChunkSpec<'spec>,
    execution: TransferExecution<'a, W>,
) -> anyhow::Result<ChunkResult> {
    let opened = match opened::send(spec, execution.cancel, execution.traffic).await? {
        opened::Opened::Response(opened) => opened,
        opened::Opened::CancelledBeforeRequest => return Ok(outcome::cancelled_before_request()),
        opened::Opened::CancelledAfterRequest => return Ok(outcome::cancelled_after_request()),
    };
    execution.traffic.opened(opened.ttfb);
    let (input, reply) = prepare(spec, execution, opened.response)?;
    dispatch(input, reply).await
}

fn prepare<'a, 'spec, W: ChunkWrite + ?Sized>(
    spec: &'a ChunkSpec<'spec>,
    execution: TransferExecution<'a, W>,
    response: ghostr_net::media_request_executor::MediaResponse,
) -> anyhow::Result<(TransferInput<'a, 'spec, W>, ResponseReply)> {
    let length = response.content_length();
    let reply = classify(&response, spec.request, spec.continuation.is_some())?;
    let total = reply::total(&reply, length);
    let evidence = HttpResponseEvidence::from_response(&response);
    let generation = OriginGeneration::from_response(&response, total)?;
    Ok((
        TransferInput {
            response,
            spec,
            sink: execution.sink,
            cancel: execution.cancel,
            network: execution.network,
            traffic: execution.traffic,
            generation,
            length,
            evidence,
        },
        reply,
    ))
}

struct TransferInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    response: ghostr_net::media_request_executor::MediaResponse,
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
    let returned = reply::range_spec(input.spec, range);
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
    let mode = reply::response_mode(request);
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
    args.execute().await
}

impl<'a, 'spec, W: ChunkWrite + ?Sized> ReceiveArgs<'a, 'spec, W> {
    async fn execute(self) -> anyhow::Result<ChunkResult> {
        let input = self.input;
        let returned = self.returned;
        streamed::receive(streamed::ReceiveInput {
            response: input.response,
            spec: &returned,
            generation: &input.generation,
            sink: input.sink,
            cancel: input.cancel,
            network: input.network,
            traffic: input.traffic,
            total: self.total,
            range_support: self.range_support,
            range_ignored: false,
            promoted: self.promoted,
            mode: self.mode,
            observation: self.observation,
            evidence: input.evidence,
        })
        .await
    }
}
