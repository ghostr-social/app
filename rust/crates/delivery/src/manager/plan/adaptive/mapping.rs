use super::super::PlannedTransfer;
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{Allocation, ControlMode, PlannerCommand, WarpPlanningDecision};
use ghostr_engine::origin_model::OriginAttemptProfile;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::{HashMap, HashSet};

pub(super) fn selected_transfers(
    state: &DeliveryState,
    present: &HashMap<PostId, Vec<ByteRange>>,
    decision: &WarpPlanningDecision,
    mode: ControlMode,
) -> Vec<PlannedTransfer> {
    let selected = decision.selected.as_ref();
    let allocation = match selected.map(|item| &item.command) {
        Some(PlannerCommand::Transfer(allocation)) => allocation,
        Some(PlannerCommand::Hedge { transfer, .. }) => transfer,
        _ => return Vec::new(),
    };
    let profile = selected.and_then(|item| {
        item.node.request_profile().map(|profile| {
            OriginAttemptProfile::new(profile)
                .with_admission_intent(item.node.origin_admission_intent())
        })
    });
    profile
        .and_then(|item| transfer(state, present, allocation, mode, item))
        .into_iter()
        .collect()
}

pub(super) fn retained_actions(
    in_flight: &[ActiveAction],
    decision: &WarpPlanningDecision,
) -> HashSet<ghostr_engine::ActionId> {
    let mut retained: HashSet<_> = in_flight.iter().map(ActiveAction::action_id).collect();
    match decision.selected.as_ref().map(|item| &item.command) {
        Some(PlannerCommand::Cancel(action)) => {
            retained.remove(action);
        }
        Some(PlannerCommand::Transfer(_)) => {
            for action in decision.generated.aborted_action_ids() {
                retained.remove(&action);
            }
        }
        _ => {}
    }
    retained
}

pub(super) fn retained_posts(
    in_flight: &[ActiveAction],
    retained: &HashSet<ghostr_engine::ActionId>,
) -> HashSet<PostId> {
    in_flight
        .iter()
        .filter(|active| retained.contains(&active.action_id()))
        .map(|active| active.post().clone())
        .collect()
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
    mode: ControlMode,
    profile: OriginAttemptProfile,
) -> Option<PlannedTransfer> {
    let identity = state
        .catalog()
        .transfer_identity(&allocation.post, &allocation.source)?;
    Some(PlannedTransfer {
        control_mode: mode,
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
        profile,
        retrieval: allocation.request,
        commitment_until_ms: allocation.commitment_until_ms,
    })
}
