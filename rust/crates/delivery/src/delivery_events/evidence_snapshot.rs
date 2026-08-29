use super::plan_evidence::PlanEvidencePage;
use super::{DecisionLog, DeliveryHandle, PlanEvidence};
use crate::evaluation::EvaluationSnapshot;
use ghostr_engine::adaptive::{AllocationPlan, DecisionRecord, DecisionReplayStatus};
use ghostr_engine::origin_model::NetworkClass;
use serde::Serialize;

const SCHEMA_VERSION: u16 = 1;
const MAX_PLAN_SNAPSHOTS: usize = 64;

#[derive(Serialize)]
struct DeliveryEvidencePage {
    schema_version: u16,
    plan_page: PlanEvidencePageSnapshot,
    evaluation: EvaluationSnapshot,
}

#[derive(Serialize)]
struct DecisionEvidenceSnapshot {
    schema_version: u16,
    decisions: super::DecisionHistorySnapshot,
    integrity: Vec<DecisionIntegrityEvidence>,
}

#[derive(Serialize)]
struct DecisionIntegrityEvidence {
    sequence: u64,
    status: DecisionReplayStatus,
    search_status: DecisionReplayStatus,
}

#[derive(Serialize)]
struct PlanEvidencePageSnapshot {
    oldest_retained_revision: u64,
    latest_retained_revision: u64,
    cursor_truncated: bool,
    has_more: bool,
    records: Vec<PlanEvidenceSnapshot>,
}

#[derive(Serialize)]
struct PlanEvidenceSnapshot {
    revision: u64,
    decision_sequence: Option<u64>,
    observed_at_ms: u64,
    current_post_id: Option<String>,
    focus_generation: Option<u64>,
    focus_covers_from: Option<u64>,
    network_status_generation: u64,
    network_class: NetworkClass,
    network_profile_generation: u64,
    plan: AllocationPlan,
}

impl DeliveryHandle {
    /// Serializes a bounded page of planning and evaluation evidence.
    ///
    /// # Errors
    ///
    /// Returns an error when the captured evidence cannot be serialized as JSON.
    pub fn evidence_page_json(
        &self,
        after_revision: u64,
        plan_limit: usize,
    ) -> serde_json::Result<String> {
        let limit = plan_limit.min(MAX_PLAN_SNAPSHOTS);
        let page = self.plans.page(after_revision, limit);
        serde_json::to_string(&DeliveryEvidencePage {
            schema_version: SCHEMA_VERSION,
            plan_page: PlanEvidencePageSnapshot::capture(&page, &self.decisions),
            evaluation: self.evaluation.snapshot(),
        })
    }

    /// Serializes the retained decision history.
    ///
    /// # Errors
    ///
    /// Returns an error when the retained history cannot be serialized as JSON.
    pub fn decision_history_json(&self) -> serde_json::Result<String> {
        let decisions = self.decisions.snapshot();
        let integrity = decisions
            .records
            .iter()
            .map(DecisionIntegrityEvidence::capture)
            .collect();
        serde_json::to_string(&DecisionEvidenceSnapshot {
            schema_version: SCHEMA_VERSION,
            decisions,
            integrity,
        })
    }
}

impl DecisionIntegrityEvidence {
    fn capture(record: &DecisionRecord) -> Self {
        Self {
            sequence: record.sequence,
            status: record.integrity_status(),
            search_status: record.search_integrity_status(),
        }
    }
}

impl PlanEvidencePageSnapshot {
    fn capture(value: &PlanEvidencePage, privacy: &DecisionLog) -> Self {
        Self {
            oldest_retained_revision: value.oldest_retained_revision,
            latest_retained_revision: value.latest_retained_revision,
            cursor_truncated: value.cursor_truncated,
            has_more: value.has_more,
            records: value
                .records
                .iter()
                .map(|plan| PlanEvidenceSnapshot::capture(plan, privacy))
                .collect(),
        }
    }
}

impl PlanEvidenceSnapshot {
    fn capture(value: &PlanEvidence, privacy: &DecisionLog) -> Self {
        Self {
            revision: value.revision,
            decision_sequence: value.decision_sequence,
            observed_at_ms: value.observed_at_ms,
            current_post_id: value
                .current
                .as_ref()
                .map(|post| privacy.pseudonymized_post(post.as_str())),
            focus_generation: value.focus_generation,
            focus_covers_from: value.focus_covers_from,
            network_status_generation: value.network_status_generation,
            network_class: value.network_class,
            network_profile_generation: value.network_profile_generation,
            plan: privacy.sanitized_plan(&value.plan),
        }
    }
}
