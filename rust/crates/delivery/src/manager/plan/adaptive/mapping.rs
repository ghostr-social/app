use super::super::PlannedTransfer;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::AllocationPlan;
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
        .filter_map(|allocation| {
            let identity = state
                .catalog()
                .transfer_identity(&allocation.post, &allocation.source)?;
            Some(PlannedTransfer {
                request: RangeRequest {
                    chunk: ChunkId {
                        post: allocation.post.clone(),
                        range: allocation.range,
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
                commitment_until_ms: allocation.commitment_until_ms,
            })
        })
        .collect()
}

pub(super) fn retained_transfers(
    state: &DeliveryState,
    plan: &AllocationPlan,
) -> HashSet<super::super::PlannedTransferId> {
    plan.retained
        .iter()
        .filter_map(|work| {
            let identity = state
                .catalog()
                .transfer_identity(&work.post, &work.source)?;
            Some(super::super::PlannedTransferId {
                chunk: ChunkId {
                    post: work.post.clone(),
                    range: work.range,
                },
                identity,
            })
        })
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
