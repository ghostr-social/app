use super::plan::PlannedTransfer;
use ghostr_engine::adaptive::{ControlMode, RetrievalRequest};
use ghostr_engine::origin_model::{
    Admission, DecisionMode, NetworkClass, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;

#[cfg(test)]
#[path = "origin_admission/capped_request_profile_test.rs"]
mod capped_request_profile_test;
#[cfg(test)]
#[path = "origin_admission/mode_test.rs"]
mod mode_test;
#[cfg(test)]
#[path = "origin_admission/request_profile_test.rs"]
mod request_profile_test;

pub(super) fn query(
    transfer: &PlannedTransfer,
    observed_at_ms: u64,
    concurrency: usize,
    network_class: NetworkClass,
) -> OriginQuery {
    OriginQuery::new(
        transfer.url.clone(),
        transfer
            .profile
            .request()
            .context()
            .with_network(network_class)
            .with_concurrency(concurrency)
            .with_observed_at_ms(observed_at_ms),
    )
}

pub(super) fn mode(transfer: &PlannedTransfer) -> DecisionMode {
    match transfer.control_mode {
        ControlMode::Emergency => DecisionMode::Emergency,
        ControlMode::Safety => DecisionMode::Safety,
        ControlMode::Normal => DecisionMode::Normal,
    }
}

pub(super) fn apply(
    mut transfer: PlannedTransfer,
    admission: &Admission,
) -> Option<PlannedTransfer> {
    let maximum = match admission {
        Admission::Production | Admission::Exploration | Admission::RecoveryTrial => {
            return Some(transfer);
        }
        Admission::RecoveryProbe { maximum_bytes } => *maximum_bytes,
        Admission::Blocked => return None,
    };
    let method = capped_method(&transfer.retrieval, transfer.profile.request().method());
    transfer.retrieval = cap_request(transfer.retrieval, maximum);
    transfer.request.chunk.range = transfer.retrieval.requested_bytes();
    transfer.profile = transfer
        .profile
        .with_executed_transport(method, transfer.retrieval.requested_bytes().len());
    Some(transfer)
}

fn capped_method(request: &RetrievalRequest, planned: RequestMethod) -> RequestMethod {
    match request {
        RetrievalRequest::FetchWhole { .. } => RequestMethod::RangeGet,
        RetrievalRequest::FetchRange { .. } => planned,
    }
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
