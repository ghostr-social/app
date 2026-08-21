use crate::RequestAuthority;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReserveConstraint {
    pub reserved_request_slots: u16,
    pub reserved_network_bytes: u64,
    pub reserved_storage_bytes: u64,
    pub reserved_cpu_ms: u64,
    pub global_request_width: u16,
    pub authority_occupancy: Vec<ReserveAuthorityOccupancy>,
    pub protected_action_ids: Vec<u16>,
    pub chance: Option<RescueChanceEvidence>,
    pub degraded: bool,
    pub degraded_reason: Option<ReserveDegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveAuthorityOccupancy {
    pub authority: RequestAuthority,
    pub occupied_request_slots: usize,
    pub request_width: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RescueChanceEvidence {
    pub deadline_ms: u64,
    pub threshold_bps: u16,
    pub achieved_success_bps: u16,
    pub transport_success_bps: u16,
    pub timing_quantile: RescueTimingQuantile,
    pub timing_completion_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RescueTimingQuantile {
    P95,
    P99,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReserveDegradedReason {
    NoFeasibleRescue,
    ProtectionFailed,
}
