use super::plan::PlannedTransfer;
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::origin_model::{
    Admission, DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;

pub(super) fn query(
    transfer: &PlannedTransfer,
    observed_at_ms: u64,
    concurrency: usize,
    network_class: NetworkClass,
) -> OriginQuery {
    let request = transfer.retrieval;
    OriginQuery::new(
        transfer.url.clone(),
        OriginContext::new(
            method(request),
            request.requested_bytes().len(),
            media(request),
        )
        .with_network(network_class)
        .with_concurrency(concurrency)
        .with_observed_at_ms(observed_at_ms),
    )
}

pub(super) fn mode(transfer: &PlannedTransfer) -> DecisionMode {
    match transfer.request.authority {
        PreemptionAuthority::PlaybackCritical => DecisionMode::Emergency,
        PreemptionAuthority::Transition => DecisionMode::Safety,
        PreemptionAuthority::Speculative => DecisionMode::Normal,
    }
}

pub(super) fn apply(
    mut transfer: PlannedTransfer,
    admission: Admission,
) -> Option<PlannedTransfer> {
    let maximum = match admission {
        Admission::Production => return Some(transfer),
        Admission::Exploration { maximum_bytes } | Admission::RecoveryProbe { maximum_bytes } => {
            maximum_bytes
        }
        Admission::Blocked => return None,
    };
    transfer.retrieval = cap_request(transfer.retrieval, maximum);
    transfer.request.chunk.range = transfer.retrieval.requested_bytes();
    Some(transfer)
}

pub(crate) fn cap_request(request: RetrievalRequest, maximum: u64) -> RetrievalRequest {
    let requested = request.requested_bytes();
    let start = match request {
        RetrievalRequest::FetchWhole { .. } => 0,
        RetrievalRequest::FetchRange { .. } => requested.start,
    };
    let end = start.saturating_add(maximum.max(1)).min(requested.end);
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(start, end),
        promotion: None,
    }
}

fn method(request: RetrievalRequest) -> RequestMethod {
    match request {
        RetrievalRequest::FetchRange { .. } => RequestMethod::RangeGet,
        RetrievalRequest::FetchWhole { .. } => RequestMethod::FullGet,
    }
}

fn media(request: RetrievalRequest) -> MediaClass {
    match request {
        RetrievalRequest::FetchRange { .. } => MediaClass::ProgressiveMp4,
        RetrievalRequest::FetchWhole { .. } => MediaClass::WholeObject,
    }
}
