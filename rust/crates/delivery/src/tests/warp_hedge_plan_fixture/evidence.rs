use super::OBSERVED_AT_MS;
use crate::manager::inflight::ActiveAction;
use crate::manager::plan::{planned_work, PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

pub(super) struct PlanEvidence {
    retry: RetryBook,
    present: HashMap<PostId, Vec<ByteRange>>,
    finalized: HashSet<PostId>,
    totals: HashMap<PostId, u64>,
    strings: HashMap<PostId, String>,
    string_sets: HashMap<PostId, HashSet<String>>,
    completed_probes: HashSet<PostId>,
    demanded: HashMap<PostId, ByteRange>,
}

impl PlanEvidence {
    pub(super) fn new(post: PostId) -> Self {
        Self {
            retry: RetryBook::new(RetryPolicy::default()),
            present: HashMap::from([(post.clone(), vec![ByteRange::new(64_000, 1_000_000)])]),
            finalized: HashSet::new(),
            totals: HashMap::from([(post, 1_000_000)]),
            strings: HashMap::new(),
            string_sets: HashMap::new(),
            completed_probes: HashSet::new(),
            demanded: HashMap::new(),
        }
    }

    pub(super) fn plan(
        &self,
        state: &mut DeliveryState,
        stats: &ghostr_engine::host_stats::HostStats,
        active: &[ActiveAction],
    ) -> PlannedWork {
        let revisions = HashMap::new();
        planned_work(
            state,
            PlanInputs {
                stats,
                retry: &self.retry,
                present: &self.present,
                finalized: &self.finalized,
                stored_totals: &self.totals,
                continuation_sources: &self.strings,
                revisions: &revisions,
                independent_sources: &self.string_sets,
                completed_head_probes: &self.completed_probes,
                in_flight: active,
                active_head_probes: &[],
                storage: StorageSnapshot::new(2_000_000, 936_000),
                connection_capacity: 3,
                connection_ceiling: 3,
                per_authority_request_limit: 2,
                packet_loss_bps: 0,
                observed_at_ms: OBSERVED_AT_MS,
                demanded: &self.demanded,
            },
        )
    }
}
