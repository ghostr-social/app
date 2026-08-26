use super::{ActiveChunk, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use core::cmp::Ordering;
use ghostr_engine::scheduling::{compare, RangeRequest};
use ghostr_engine::{ActionId, ChunkId, PostId};
use std::collections::HashSet;

impl InFlightChunks {
    pub fn reconcile_with_commitments(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        retained: &HashSet<ActionId>,
    ) {
        self.cancel_unplanned(planned, retained);
        self.reserve_for_missing(planned, capacity.max(1));
    }

    pub(crate) fn contains_transfer(&self, transfer: &PlannedTransfer) -> bool {
        self.transfers
            .values()
            .any(|active| !active.cancelling && action_matches(active, transfer))
    }

    fn cancel_unplanned(&mut self, planned: &[PlannedTransfer], retained: &HashSet<ActionId>) {
        for (action, active) in &mut self.transfers {
            active.policy_retained = false;
            if active.cancelling {
                continue;
            }
            let current = planned
                .iter()
                .find(|transfer| action_matches(active, transfer));
            if let Some(transfer) = current {
                if active.priority.authority != transfer.request.authority {
                    active.priority = transfer.request.clone();
                }
                continue;
            }
            let committed = retained.contains(action);
            active.policy_retained = committed;
            if !committed && !active.io_finished() {
                active.cancel();
            }
        }
    }

    fn reserve_for_missing(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        let mut reserved = 0;
        for transfer in planned {
            if self.contains_transfer(transfer) {
                continue;
            }
            self.reserve_one(transfer, capacity, &mut reserved);
        }
    }

    fn reserve_one(&mut self, transfer: &PlannedTransfer, capacity: usize, reserved: &mut usize) {
        if self.len().saturating_add(*reserved) >= capacity {
            if self.transfers.values().any(ActiveChunk::io_finished) {
                return;
            }
            let Some(victim) = self.lowest_victim(&transfer.request) else {
                return;
            };
            self.cancel_action(victim);
            return;
        }
        *reserved = reserved.saturating_add(1);
    }

    fn lowest_victim(&self, request: &RangeRequest) -> Option<ActionId> {
        self.transfers
            .iter()
            .filter(|(_, active)| can_yield(active, request))
            .max_by(|left, right| request_order(&left.1.priority, &right.1.priority))
            .map(|(action, _)| *action)
    }

    pub fn preempt_for_current(&mut self, current: &PostId, priority: &[ChunkId], capacity: usize) {
        if self.transfers.values().any(ActiveChunk::io_finished) {
            return;
        }
        let Some(rank) = self.preemption_rank(current, priority, capacity) else {
            return;
        };
        while self.len() >= capacity {
            let Some(victim) = self.lower_priority_victim(current, &priority[rank + 1..]) else {
                return;
            };
            self.cancel_action(victim);
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

    fn lower_priority_victim(&self, current: &PostId, priority: &[ChunkId]) -> Option<ActionId> {
        priority.iter().rev().find_map(|request| {
            self.transfers.iter().find_map(|(action, active)| {
                (!active.io_finished()
                    && !active.cancelling
                    && !active.policy_retained
                    && &active.chunk.post != current
                    && covers(&active.chunk, request))
                .then_some(*action)
            })
        })
    }
}

fn can_yield(active: &ActiveChunk, request: &RangeRequest) -> bool {
    !active.io_finished()
        && !active.cancelling
        && !active.policy_retained
        && request_order(&active.priority, request).is_gt()
}

fn action_matches(active: &ActiveChunk, transfer: &PlannedTransfer) -> bool {
    active.identity == transfer.identity
        && retrieval_matches(active.effective_request, transfer.retrieval)
}

fn retrieval_matches(
    active: ghostr_engine::adaptive::RetrievalRequest,
    planned: ghostr_engine::adaptive::RetrievalRequest,
) -> bool {
    match (active, planned) {
        (
            ghostr_engine::adaptive::RetrievalRequest::FetchRange {
                bytes: active,
                promotion: None,
            },
            ghostr_engine::adaptive::RetrievalRequest::FetchRange {
                bytes: planned,
                promotion: None,
            },
        ) => active.start < planned.end && planned.start < active.end,
        _ => active == planned,
    }
}

fn request_order(left: &RangeRequest, right: &RangeRequest) -> Ordering {
    compare(left, right)
}

fn covers(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start <= request.range.start
        && active.range.end >= request.range.end
}

#[cfg(test)]
#[path = "reconciliation_axiom_test.rs"]
pub(crate) mod axiom_test_support;
