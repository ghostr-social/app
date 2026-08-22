use crate::delivery_events::DecisionResolution;
use ghostr_engine::adaptive::{
    RecordedAllocationReason, RecordedRetrievalRequest, RecordedTransfer, RecordedWarpCommand,
};

pub(super) fn is_whole(value: &DecisionResolution) -> bool {
    match transfer(value) {
        Some(transfer) => matches!(
            transfer.request,
            RecordedRetrievalRequest::FetchWhole { .. }
        ),
        None => value.action.request == "whole",
    }
}

pub(super) fn is_probe(value: &DecisionResolution) -> bool {
    match transfer(value) {
        Some(transfer) => matches!(
            transfer.reason,
            RecordedAllocationReason::MediaBootstrap
                | RecordedAllocationReason::MediaLayoutDiscovery
        ),
        None => matches!(
            value.action.reason.as_str(),
            "MediaBootstrap" | "MediaLayoutDiscovery"
        ),
    }
}

fn transfer(value: &DecisionResolution) -> Option<&RecordedTransfer> {
    match &value.warp_action.as_ref()?.command {
        RecordedWarpCommand::Transfer { transfer }
        | RecordedWarpCommand::Hedge { transfer, .. } => Some(transfer),
        _ => None,
    }
}
