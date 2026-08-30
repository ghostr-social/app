use crate::manager::plan::axiom_test_support::planned_work;
use super::OBSERVED_AT_MS;
use crate::manager::inflight::ActiveAction;
use crate::manager::plan::{PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{StorageSnapshot, WholeBodyExhaustion};
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

mod identity;
pub(super) use identity::learn_identity;

pub(super) struct PlanEvidence {
    retry: RetryBook,
    present: HashMap<PostId, Vec<ByteRange>>,
    finalized: HashSet<PostId>,
    totals: HashMap<PostId, u64>,
    strings: HashMap<PostId, String>,
    string_sets: HashMap<PostId, HashSet<String>>,
    completed_probes: HashSet<TransferIdentity>,
    exhausted_caps: HashMap<TransferIdentity, WholeBodyExhaustion>,
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
            exhausted_caps: HashMap::new(),
            demanded: HashMap::new(),
        }
    }

    pub(super) fn plan(
        &self,
        state: &DeliveryState,
        stats: &ghostr_engine::host_stats::HostStats,
        active: &[ActiveAction],
    ) -> PlannedWork {
        let revisions = HashMap::new();
        planned_work(
            state,
            &PlanInputs {
                stats,
                retry: &self.retry,
                present: &self.present,
                finalized: &self.finalized,
                stored_totals: &self.totals,
                continuation_sources: &self.strings,
                revisions: &revisions,
                independent_sources: &self.string_sets,
                whole_body_exhaustions: &self.exhausted_caps,
                completed_head_probes: &self.completed_probes,
                unavailable_head_probes: &HashSet::new(),
                in_flight: active,
                active_head_probes: &[],
                hls_candidates: &[],
                active_hls_sources: &[],
                segmented_storage_available_bytes: u64::MAX,
                storage: StorageSnapshot::new(2_000_000, 936_000),
                connection_capacity: 3,
                hls_demand_expansion_allowed: true,
                connection_ceiling: 3,
                per_authority_request_limit: 2,
                packet_loss_bps: 0,
                resource_feedback: None,
                capacity_revision: 0,
                observed_at_ms: OBSERVED_AT_MS,
                demanded: &self.demanded,
            },
        )
    }
}
