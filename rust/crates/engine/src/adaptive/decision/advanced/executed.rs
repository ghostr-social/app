use super::command::RecordedRetrievalRequest;
use super::{RecordedResourceCost, RecordedWarpAction, RecordedWarpCommand};
use crate::adaptive::decision::executed::ExecutedRequest;
use crate::adaptive::decision::privacy::DecisionPrivacy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedExecutedRequest {
    pub post_id: String,
    pub source_id: String,
    pub request: RecordedRetrievalRequest,
    pub resources: RecordedResourceCost,
}

pub(in crate::adaptive::decision) fn capture(
    value: &ExecutedRequest,
    privacy: &DecisionPrivacy,
) -> RecordedExecutedRequest {
    RecordedExecutedRequest {
        post_id: privacy.post(value.post.as_str()),
        source_id: privacy.source(&value.source),
        request: super::command::request::capture(value.request),
        resources: value.resources.into(),
    }
}

pub(in crate::adaptive::decision) fn coherent(
    executed: &RecordedExecutedRequest,
    selected: &RecordedWarpAction,
) -> bool {
    let Some(transfer) = transfer(&selected.command) else {
        return false;
    };
    executed.post_id == transfer.post_id
        && executed.source_id == transfer.source_id
        && request_subset(executed.request, transfer.request)
        && exact_resources(executed)
        && fits(executed.resources, selected.resources)
}

fn transfer(command: &RecordedWarpCommand) -> Option<&super::RecordedTransfer> {
    match command {
        RecordedWarpCommand::Transfer { transfer }
        | RecordedWarpCommand::Hedge { transfer, .. } => Some(transfer),
        _ => None,
    }
}

fn request_subset(executed: RecordedRetrievalRequest, intent: RecordedRetrievalRequest) -> bool {
    if executed == intent {
        return true;
    }
    let RecordedRetrievalRequest::FetchRange {
        bytes_start,
        bytes_end,
        promotion: None,
    } = executed
    else {
        return false;
    };
    let (intent_start, intent_end) = intent.bytes();
    bytes_start < bytes_end && intent_start <= bytes_start && bytes_end <= intent_end
}

fn exact_resources(executed: &RecordedExecutedRequest) -> bool {
    let (start, end) = executed.request.bytes();
    let bytes = end.saturating_sub(start);
    executed.resources
        == RecordedResourceCost {
            network_bytes: bytes,
            storage_bytes: bytes,
            cpu_ms: 0,
            requests: 1,
        }
}

fn fits(cost: RecordedResourceCost, limit: RecordedResourceCost) -> bool {
    cost.network_bytes <= limit.network_bytes
        && cost.storage_bytes <= limit.storage_bytes
        && cost.cpu_ms <= limit.cpu_ms
        && cost.requests <= limit.requests
}
