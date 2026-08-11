use super::{overlaps, ActiveChunk, InFlightChunks};
use crate::manager::plan::eviction::ProtectedSeedEviction;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scoring::{compare, ChunkRequest};
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ChunkId, PostId};
use std::{cmp::Ordering, collections::HashSet};

impl InFlightChunks {
    /// Retains planned IO, then reserves slots for higher-priority work.
    #[cfg(test)]
    pub fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_eviction(planned, capacity, ProtectedSeedEviction::Allow);
    }

    #[cfg(test)]
    pub fn reconcile_with_eviction(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        eviction: ProtectedSeedEviction,
    ) {
        self.reconcile_with_commitments(planned, capacity, eviction, &HashSet::new());
    }

    pub fn reconcile_with_commitments(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        eviction: ProtectedSeedEviction,
        protected_identities: &HashSet<TransferIdentity>,
    ) {
        self.cancel_unplanned(planned, protected_identities);
        self.reserve_for_missing(planned, capacity.max(1), eviction);
    }

    fn cancel_unplanned(
        &mut self,
        planned: &[PlannedTransfer],
        protected_identities: &HashSet<TransferIdentity>,
    ) {
        self.transfers.retain(|chunk, active| {
            let finished = active.io_finished();
            let current = planned.iter().find(|transfer| {
                transfer.identity == active.identity && overlaps(chunk, &transfer.request.chunk)
            });
            if let Some(transfer) = current {
                if active.request.tier != transfer.request.tier {
                    active.request = transfer.request.clone();
                }
                return true;
            }
            let foreground_replanned = planned.iter().any(|transfer| {
                transfer.identity == active.identity
                    && transfer.request.chunk.range.start != chunk.range.end
                    && matches!(
                        transfer.request.tier,
                        Tier::T0PlaybackEmergency | Tier::T1CurrentTail
                    )
            });
            let committed = active.started_as_seed
                && !foreground_replanned
                && protected_identities.contains(&active.identity);
            if !finished && !committed {
                active.handle.cancel();
            }
            finished || committed
        });
    }

    fn reserve_for_missing(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        eviction: ProtectedSeedEviction,
    ) {
        let mut reserved = 0;
        for transfer in planned {
            if self.contains(&transfer.request.chunk) {
                continue;
            }
            self.reserve_one(&transfer.request, capacity, &mut reserved, eviction);
        }
    }

    fn reserve_one(
        &mut self,
        request: &ChunkRequest,
        capacity: usize,
        reserved: &mut usize,
        eviction: ProtectedSeedEviction,
    ) {
        while self.len().saturating_add(*reserved) >= capacity {
            let Some(victim) = self.lowest_victim(request, eviction) else {
                return;
            };
            self.cancel(&victim);
        }
        *reserved = reserved.saturating_add(1);
    }

    fn lowest_victim(
        &self,
        request: &ChunkRequest,
        eviction: ProtectedSeedEviction,
    ) -> Option<ChunkId> {
        self.transfers
            .iter()
            .filter(|(_, active)| can_yield(active, request, eviction))
            .max_by(|left, right| request_order(&left.1.request, &right.1.request))
            .map(|(chunk, _)| chunk.clone())
    }

    #[cfg(test)]
    pub fn preempt_for_current(&mut self, current: &PostId, priority: &[ChunkId], capacity: usize) {
        self.preempt_for_current_with_eviction(
            current,
            priority,
            capacity,
            ProtectedSeedEviction::Allow,
        );
    }

    pub fn preempt_for_current_with_eviction(
        &mut self,
        current: &PostId,
        priority: &[ChunkId],
        capacity: usize,
        eviction: ProtectedSeedEviction,
    ) {
        let Some(rank) = self.preemption_rank(current, priority, capacity) else {
            return;
        };
        while self.len() >= capacity {
            let Some(victim) = self.lower_priority_victim(current, &priority[rank + 1..], eviction)
            else {
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

    fn lower_priority_victim(
        &self,
        current: &PostId,
        priority: &[ChunkId],
        eviction: ProtectedSeedEviction,
    ) -> Option<ChunkId> {
        priority.iter().rev().find_map(|request| {
            self.transfers.iter().find_map(|(chunk, active)| {
                (!active.io_finished()
                    && !protected_seed(active, eviction)
                    && &chunk.post != current
                    && covers(chunk, request))
                .then(|| chunk.clone())
            })
        })
    }
}

fn can_yield(
    active: &ActiveChunk,
    request: &ChunkRequest,
    eviction: ProtectedSeedEviction,
) -> bool {
    !active.io_finished()
        && !protected_seed(active, eviction)
        && !matches!(
            (active.request.tier, request.tier),
            (Tier::T2Startability, Tier::T2Startability)
        )
        && request_order(&active.request, request).is_gt()
}

fn protected_seed(active: &ActiveChunk, eviction: ProtectedSeedEviction) -> bool {
    eviction == ProtectedSeedEviction::Defer && active.request.tier == Tier::T2Startability
}

fn request_order(left: &ChunkRequest, right: &ChunkRequest) -> Ordering {
    compare(left, right)
}

fn covers(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start <= request.range.start
        && active.range.end >= request.range.end
}
