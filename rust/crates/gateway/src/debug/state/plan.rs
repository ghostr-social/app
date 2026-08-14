use ghostr_delivery::delivery_events::PlanEvidence;
use ghostr_engine::adaptive::{
    Allocation, AllocationReason, CandidateUtility, DiscoveryDemand, Eviction, EvictionReason,
    PreemptionAuthority, RetainedAllocation,
};
use ghostr_engine::ByteRange;
use serde::Serialize;

mod next_reserve;
use next_reserve::{snapshot as next_reserve, NextReserveSnapshot};

#[derive(Debug, Serialize)]
pub(super) struct AdaptivePlanSnapshot {
    revision: u64,
    observed_at_ms: u64,
    discovery_demand: &'static str,
    next_reserve: NextReserveSnapshot,
    allocations: Vec<AllocationSnapshot>,
    retained: Vec<RetainedSnapshot>,
    evictions: Vec<EvictionSnapshot>,
}

#[derive(Debug, Serialize)]
struct AllocationSnapshot {
    post_id: String,
    range: RangeSnapshot,
    source: String,
    expected_playable_gain_ms: u64,
    utility: UtilitySnapshot,
    authority: &'static str,
    commitment_until_ms: u64,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
struct UtilitySnapshot {
    view_probability: f64,
    additional_playable_ms: u64,
    expected_delivery_ms: u64,
    score: f64,
}

#[derive(Debug, Serialize)]
struct RetainedSnapshot {
    post_id: String,
    range: RangeSnapshot,
    source: String,
    utility: UtilitySnapshot,
    committed_until_ms: u64,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
struct EvictionSnapshot {
    post_id: String,
    range: RangeSnapshot,
    expected_playable_loss_ms: f64,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct RangeSnapshot {
    start: u64,
    end: u64,
}

pub(super) fn snapshots(history: &[PlanEvidence]) -> Vec<AdaptivePlanSnapshot> {
    history.iter().map(snapshot).collect()
}

fn snapshot(evidence: &PlanEvidence) -> AdaptivePlanSnapshot {
    AdaptivePlanSnapshot {
        revision: evidence.revision,
        observed_at_ms: evidence.observed_at_ms,
        discovery_demand: demand(evidence.plan.discovery_demand),
        next_reserve: next_reserve(&evidence.plan.next_reserve),
        allocations: evidence.plan.allocations.iter().map(allocation).collect(),
        retained: evidence.plan.retained.iter().map(retained).collect(),
        evictions: evidence.plan.evictions.iter().map(eviction).collect(),
    }
}

fn allocation(value: &Allocation) -> AllocationSnapshot {
    AllocationSnapshot {
        post_id: value.post.as_str().to_owned(),
        range: range(value.range),
        source: value.source.clone(),
        expected_playable_gain_ms: value.expected_playable_gain_ms,
        utility: utility(value.utility),
        authority: authority(value.authority),
        commitment_until_ms: value.commitment_until_ms,
        reason: allocation_reason(value.reason),
    }
}

fn utility(value: CandidateUtility) -> UtilitySnapshot {
    UtilitySnapshot {
        view_probability: value.view_probability,
        additional_playable_ms: value.additional_playable_ms,
        expected_delivery_ms: value.expected_delivery_ms,
        score: value.score,
    }
}

fn retained(value: &RetainedAllocation) -> RetainedSnapshot {
    RetainedSnapshot {
        post_id: value.post.as_str().to_owned(),
        range: range(value.range),
        source: value.source.clone(),
        utility: utility(value.utility),
        committed_until_ms: value.committed_until_ms,
        reason: allocation_reason(value.reason),
    }
}

fn eviction(value: &Eviction) -> EvictionSnapshot {
    EvictionSnapshot {
        post_id: value.post.as_str().to_owned(),
        range: range(value.range),
        expected_playable_loss_ms: value.expected_playable_loss_ms,
        reason: eviction_reason(value.reason),
    }
}

pub(super) fn range(value: ByteRange) -> RangeSnapshot {
    RangeSnapshot {
        start: value.start,
        end: value.end,
    }
}

fn demand(value: DiscoveryDemand) -> &'static str {
    match value {
        DiscoveryDemand::Expand => "expand",
        DiscoveryDemand::Hold => "hold",
    }
}

fn authority(value: PreemptionAuthority) -> &'static str {
    match value {
        PreemptionAuthority::PlaybackCritical => "playback_critical",
        PreemptionAuthority::Transition => "transition",
        PreemptionAuthority::Speculative => "speculative",
    }
}

fn allocation_reason(value: AllocationReason) -> &'static str {
    match value {
        AllocationReason::CurrentStallPrevention => "current_stall_prevention",
        AllocationReason::CurrentBufferReserve => "current_buffer_reserve",
        AllocationReason::LikelyNextTransition => "likely_next_transition",
        AllocationReason::RapidNavigationCoverage => "rapid_navigation_coverage",
        AllocationReason::MediaBootstrap => "media_bootstrap",
        AllocationReason::MediaLayoutDiscovery => "media_layout_discovery",
        AllocationReason::NextStartability => "next_startability",
        AllocationReason::UsefulCommitment => "useful_commitment",
    }
}

fn eviction_reason(value: EvictionReason) -> &'static str {
    match value {
        EvictionReason::StoragePressure => "storage_pressure",
    }
}
