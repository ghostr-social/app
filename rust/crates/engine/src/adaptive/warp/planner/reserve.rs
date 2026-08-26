use crate::RequestAuthority;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReserveConstraint {
    pub(crate) reserved_request_slots: u16,
    pub(crate) reserved_network_bytes: u64,
    pub(crate) reserved_storage_bytes: u64,
    pub(crate) reserved_cpu_ms: u64,
    pub(crate) global_request_width: u16,
    pub(crate) authority_occupancy: Vec<ReserveAuthorityOccupancy>,
    pub(crate) protected_action_ids: Vec<u16>,
    pub(crate) chance: Option<RescueChanceEvidence>,
    pub(crate) degraded: bool,
    pub(crate) degraded_reason: Option<ReserveDegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveAuthorityOccupancy {
    pub(crate) authority: RequestAuthority,
    pub(crate) occupied_request_slots: usize,
    pub(crate) request_width: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RescueChanceEvidence {
    pub(crate) deadline_ms: u64,
    pub(crate) threshold_bps: u16,
    pub(crate) achieved_success_bps: u16,
    pub(crate) transport_success_bps: u16,
    pub(crate) timing_quantile: RescueTimingQuantile,
    pub(crate) timing_completion_ms: u64,
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
