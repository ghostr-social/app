use super::{overlaps, ActiveChunk, InFlightChunks};
use crate::manager::plan::{PlannedTransfer, PlannedTransferId};
use ghostr_engine::scheduling::{compare, RangeRequest};
use ghostr_engine::{ChunkId, PostId};
use std::{cmp::Ordering, collections::HashSet};

impl InFlightChunks {
    /// Retains planned IO, then reserves slots for higher-priority work.
    #[cfg(test)]
    pub fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_commitments(planned, capacity, &HashSet::new());
    }

    pub fn reconcile_with_commitments(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        retained: &HashSet<PlannedTransferId>,
    ) {
        self.cancel_unplanned(planned, retained);
        self.reserve_for_missing(planned, capacity.max(1));
    }

    fn cancel_unplanned(
        &mut self,
        planned: &[PlannedTransfer],
        retained: &HashSet<PlannedTransferId>,
    ) {
        self.transfers.retain(|chunk, active| {
            active.policy_retained = false;
            let finished = active.io_finished();
            let current = planned.iter().find(|transfer| {
                transfer.identity == active.identity && overlaps(chunk, &transfer.request.chunk)
            });
            if let Some(transfer) = current {
                if active.request.authority != transfer.request.authority {
                    active.request = transfer.request.clone();
                }
                return true;
            }
            let committed = retained.contains(&PlannedTransferId {
                chunk: chunk.clone(),
                identity: active.identity.clone(),
            });
            active.policy_retained = committed;
            if !finished && !committed {
                active.handle.cancel();
            }
            finished || committed
        });
    }

    fn reserve_for_missing(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        let mut reserved = 0;
        for transfer in planned {
            if self.contains(&transfer.request.chunk) {
                continue;
            }
            self.reserve_one(&transfer.request, capacity, &mut reserved);
        }
    }

    fn reserve_one(&mut self, request: &RangeRequest, capacity: usize, reserved: &mut usize) {
        while self.len().saturating_add(*reserved) >= capacity {
            let Some(victim) = self.lowest_victim(request) else {
                return;
            };
            self.cancel(&victim);
        }
        *reserved = reserved.saturating_add(1);
    }

    fn lowest_victim(&self, request: &RangeRequest) -> Option<ChunkId> {
        self.transfers
            .iter()
            .filter(|(_, active)| can_yield(active, request))
            .max_by(|left, right| request_order(&left.1.request, &right.1.request))
            .map(|(chunk, _)| chunk.clone())
    }

    pub fn preempt_for_current(&mut self, current: &PostId, priority: &[ChunkId], capacity: usize) {
        let Some(rank) = self.preemption_rank(current, priority, capacity) else {
            return;
        };
        while self.len() >= capacity {
            let Some(victim) = self.lower_priority_victim(current, &priority[rank + 1..]) else {
                return;
            };
            self.cancel(&victim);
        }
    }

    fn preemption_rank(
        &self,
        current: &PostId,
        priority: &[ChunkId],
        capacity: usize,
    ) -> Option<usize> {
        let _ = capacity.checked_sub(1)?;
        priority
            .iter()
            .position(|chunk| &chunk.post == current && !self.contains(chunk))
    }

    fn lower_priority_victim(&self, current: &PostId, priority: &[ChunkId]) -> Option<ChunkId> {
        priority.iter().rev().find_map(|request| {
            self.transfers.iter().find_map(|(chunk, active)| {
                (!active.io_finished()
                    && !active.policy_retained
                    && &chunk.post != current
                    && covers(chunk, request))
                .then(|| chunk.clone())
            })
        })
    }
}

fn can_yield(active: &ActiveChunk, request: &RangeRequest) -> bool {
    !active.io_finished()
        && !active.policy_retained
        && request_order(&active.request, request).is_gt()
}

fn request_order(left: &RangeRequest, right: &RangeRequest) -> Ordering {
    compare(left, right)
}

fn covers(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start <= request.range.start
        && active.range.end >= request.range.end
}
