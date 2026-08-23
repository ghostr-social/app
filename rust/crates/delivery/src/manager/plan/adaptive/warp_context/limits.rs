use super::request_capacity::RequestCapacity;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{PlannerLimits, PlayabilitySnapshot, BOOTSTRAP_DIRECT_FETCH_BYTES};

pub(super) fn resolve(
    state: &DeliveryState,
    snapshot: &PlayabilitySnapshot,
    request_capacity: &RequestCapacity,
) -> PlannerLimits {
    let (burst, rate) = network_budget(snapshot.network.throughput_bps);
    PlannerLimits {
        network_burst_bytes: burst,
        network_rate_bytes_per_second: rate,
        cpu_ms: state
            .transform_profile()
            .map_or(0, |profile| profile.limits().cpu_ms()),
        request_tokens: request_capacity.tokens,
        per_origin_requests: snapshot
            .network
            .per_authority_request_limit
            .min(u16::MAX as usize) as u16,
    }
}

pub(super) const fn network_budget(throughput_bps: u64) -> (u64, u64) {
    let rate = maximum(throughput_bps / 8, 1);
    (
        maximum(rate.saturating_mul(2), BOOTSTRAP_DIRECT_FETCH_BYTES),
        rate,
    )
}

const fn maximum(left: u64, right: u64) -> u64 {
    if left > right {
        left
    } else {
        right
    }
}
