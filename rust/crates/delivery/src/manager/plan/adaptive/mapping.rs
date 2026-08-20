use super::super::PlannedTransfer;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{Allocation, AllocationPlan, PlannerCommand, WarpPlanningDecision};
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::{HashMap, HashSet};

pub(super) fn transfers(
    state: &DeliveryState,
    present: &HashMap<PostId, Vec<ByteRange>>,
    plan: &AllocationPlan,
) -> Vec<PlannedTransfer> {
    plan.allocations
        .iter()
        .filter_map(|allocation| transfer(state, present, allocation))
        .collect()
}

pub(super) fn selected_transfers(
    state: &DeliveryState,
    present: &HashMap<PostId, Vec<ByteRange>>,
    decision: &WarpPlanningDecision,
) -> Vec<PlannedTransfer> {
    let allocation = match decision.selected.as_ref().map(|item| &item.command) {
        Some(PlannerCommand::Transfer(allocation)) => Some(allocation),
        Some(PlannerCommand::Hedge { transfer, .. }) => Some(transfer),
        _ => None,
    };
    allocation
        .and_then(|item| transfer(state, present, item))
        .into_iter()
        .collect()
}

pub(super) fn retained_actions(
    decision: &WarpPlanningDecision,
) -> HashSet<ghostr_engine::ActionId> {
    let mut retained: HashSet<_> = decision
        .generated
        .active_controls
        .iter()
        .filter(|item| item.decision != ghostr_engine::adaptive::ContinuationDecision::Abort)
        .map(|item| item.action)
        .collect();
    match decision.selected.as_ref().map(|item| &item.command) {
        Some(PlannerCommand::Promote { action, .. }) => {
            retained.insert(*action);
        }
        Some(PlannerCommand::Hedge { primary, .. }) => {
            retained.insert(*primary);
        }
        _ => {}
    }
    retained
}

fn contiguous_end(ranges: &[ByteRange]) -> u64 {
    let mut end = 0;
    for range in ghostr_engine::media_timeline::normalize(ranges.to_vec()) {
        if range.start > end {
            break;
        }
        end = end.max(range.end);
    }
    end
}

fn transfer(
    state: &DeliveryState,
    present: &HashMap<PostId, Vec<ByteRange>>,
    allocation: &Allocation,
) -> Option<PlannedTransfer> {
    let identity = state
        .catalog()
        .transfer_identity(&allocation.post, &allocation.source)?;
    Some(PlannedTransfer {
        request: RangeRequest {
            chunk: ChunkId {
                post: allocation.post.clone(),
                range: allocation.request.requested_bytes(),
            },
            authority: allocation.authority,
            score: allocation.utility.score,
            contiguous_depth_bytes: contiguous_end(
                present
                    .get(&allocation.post)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
            ),
        },
        url: allocation.source.clone(),
        identity,
        retrieval: allocation.request,
        commitment_until_ms: allocation.commitment_until_ms,
    })
}
