use super::{
    ChunkSpec, HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::chunk::response::ResponseReply;
use crate::chunk::traffic::ChunkTraffic;
use anyhow::Context as _;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_net::origin_content_type;

pub(super) fn classify(
    response: &ghostr_net::media_request_executor::MediaResponse,
    spec: &ChunkSpec<'_>,
    evidence: &HttpResponseEvidence,
    traffic: &mut dyn ChunkTraffic,
) -> anyhow::Result<ResponseReply> {
    if let Err(error) = origin_content_type::require_admissible(response.headers()) {
        observe_rejection(
            evidence.clone().provenance_only(),
            super::ResponseRejection::MediaType,
            traffic,
        );
        return Err(error).context(super::ResponseFailure::InvalidResponse);
    }
    crate::chunk::response::classify(response, spec.request, spec.continuation.is_some()).map_err(
        |error| {
            observe_rejection(
                evidence.clone().authority_only(),
                super::ResponseRejection::Semantics,
                traffic,
            );
            error.context(response_failure(spec.request))
        },
    )
}

fn response_failure(request: RetrievalRequest) -> super::ResponseFailure {
    match request {
        RetrievalRequest::FetchRange { .. } => super::ResponseFailure::RangeNoncompliant,
        RetrievalRequest::FetchWhole { .. } => super::ResponseFailure::InvalidResponse,
    }
}

fn observe_rejection(
    evidence: HttpResponseEvidence,
    rejection: super::ResponseRejection,
    traffic: &mut dyn ChunkTraffic,
) {
    traffic.response_observed(OpenedResponse::new(
        ResponseObservation::Rejected(rejection),
        None,
        ResponseWriteMode::Sparse,
        evidence,
    ));
}

pub(super) fn body_spec<'a>(spec: &ChunkSpec<'a>, request: RetrievalRequest) -> ChunkSpec<'a> {
    ChunkSpec {
        requests: spec.requests,
        url: spec.url,
        request,
        attempt_profile: spec.attempt_profile,
        priority: spec.priority,
        continuation: spec.continuation,
        timeouts: spec.timeouts,
    }
}

pub(super) fn range_spec<'a>(
    spec: &ChunkSpec<'a>,
    range: ghostr_engine::ByteRange,
) -> ChunkSpec<'a> {
    body_spec(
        spec,
        RetrievalRequest::FetchRange {
            bytes: range,
            promotion: None,
        },
    )
}

pub(super) fn response_mode(request: RetrievalRequest) -> ResponseWriteMode {
    match request {
        RetrievalRequest::FetchWhole { contract, .. } => {
            ResponseWriteMode::SingleResponse(contract)
        }
        RetrievalRequest::FetchRange { .. } => ResponseWriteMode::Sparse,
    }
}

pub(super) fn total(reply: &ResponseReply, full_length: Option<u64>) -> Option<u64> {
    match reply {
        ResponseReply::Partial { total, .. } => *total,
        ResponseReply::BoundDiscovered { total_bytes, .. } => Some(*total_bytes),
        ResponseReply::Body { .. } | ResponseReply::Ignored { .. } => full_length,
    }
}
