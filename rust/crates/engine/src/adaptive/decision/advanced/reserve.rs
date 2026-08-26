use super::super::privacy::DecisionPrivacy;
use crate::adaptive::{
    RescueChanceEvidence, RescueTimingQuantile, ReserveAuthorityOccupancy, ReserveConstraint,
    ReserveDegradedReason,
};
use serde::{Deserialize, Serialize};

mod authority;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpReserve {
    pub(crate) reserved_request_slots: u16,
    pub(crate) reserved_network_bytes: u64,
    pub(crate) degraded: bool,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub(crate) reserved_storage_bytes: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub(crate) reserved_cpu_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub(crate) global_request_width: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) authority_occupancy: Vec<RecordedReserveAuthorityOccupancy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) protected_action_ids: Vec<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) chance: Option<RecordedRescueChanceEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) degraded_reason: Option<RecordedReserveDegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedReserveAuthorityOccupancy {
    pub(crate) authority_id: String,
    pub(crate) occupied_request_slots: u64,
    pub(crate) request_width: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedRescueChanceEvidence {
    pub(crate) deadline_ms: u64,
    pub(crate) threshold_bps: u16,
    pub(crate) achieved_success_bps: u16,
    pub(crate) transport_success_bps: u16,
    pub(crate) timing_quantile: RecordedRescueTimingQuantile,
    pub(crate) timing_completion_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedRescueTimingQuantile {
    P95,
    P99,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedReserveDegradedReason {
    NoFeasibleRescue,
    ProtectionFailed,
}

pub(in crate::adaptive::decision) fn capture(
    value: &ReserveConstraint,
    privacy: &DecisionPrivacy,
) -> RecordedWarpReserve {
    RecordedWarpReserve {
        reserved_request_slots: value.reserved_request_slots,
        reserved_network_bytes: value.reserved_network_bytes,
        degraded: value.degraded,
        reserved_storage_bytes: value.reserved_storage_bytes,
        reserved_cpu_ms: value.reserved_cpu_ms,
        global_request_width: value.global_request_width,
        authority_occupancy: authority::sorted(
            value
                .authority_occupancy
                .iter()
                .map(|item| authority::capture(item, privacy))
                .collect(),
        ),
        protected_action_ids: value.protected_action_ids.clone(),
        chance: value.chance.map(RecordedRescueChanceEvidence::from),
        degraded_reason: value
            .degraded_reason
            .map(RecordedReserveDegradedReason::from),
    }
}

pub(in crate::adaptive::decision) fn restore(
    value: &RecordedWarpReserve,
) -> Option<ReserveConstraint> {
    Some(ReserveConstraint {
        reserved_request_slots: value.reserved_request_slots,
        reserved_network_bytes: value.reserved_network_bytes,
        reserved_storage_bytes: value.reserved_storage_bytes,
        reserved_cpu_ms: value.reserved_cpu_ms,
        global_request_width: value.global_request_width,
        authority_occupancy: value
            .authority_occupancy
            .iter()
            .map(authority::restore)
            .collect::<Option<Vec<_>>>()?,
        protected_action_ids: value.protected_action_ids.clone(),
        chance: value.chance.map(RescueChanceEvidence::from),
        degraded: value.degraded,
        degraded_reason: value.degraded_reason.map(ReserveDegradedReason::from),
    })
}

impl From<RescueChanceEvidence> for RecordedRescueChanceEvidence {
    fn from(value: RescueChanceEvidence) -> Self {
        Self {
            deadline_ms: value.deadline_ms,
            threshold_bps: value.threshold_bps,
            achieved_success_bps: value.achieved_success_bps,
            transport_success_bps: value.transport_success_bps,
            timing_quantile: value.timing_quantile.into(),
            timing_completion_ms: value.timing_completion_ms,
        }
    }
}

impl From<RecordedRescueChanceEvidence> for RescueChanceEvidence {
    fn from(value: RecordedRescueChanceEvidence) -> Self {
        Self {
            deadline_ms: value.deadline_ms,
            threshold_bps: value.threshold_bps,
            achieved_success_bps: value.achieved_success_bps,
            transport_success_bps: value.transport_success_bps,
            timing_quantile: value.timing_quantile.into(),
            timing_completion_ms: value.timing_completion_ms,
        }
    }
}

impl From<RescueTimingQuantile> for RecordedRescueTimingQuantile {
    fn from(value: RescueTimingQuantile) -> Self {
        match value {
            RescueTimingQuantile::P95 => Self::P95,
            RescueTimingQuantile::P99 => Self::P99,
        }
    }
}

impl From<RecordedRescueTimingQuantile> for RescueTimingQuantile {
    fn from(value: RecordedRescueTimingQuantile) -> Self {
        match value {
            RecordedRescueTimingQuantile::P95 => Self::P95,
            RecordedRescueTimingQuantile::P99 => Self::P99,
        }
    }
}

impl From<ReserveDegradedReason> for RecordedReserveDegradedReason {
    fn from(value: ReserveDegradedReason) -> Self {
        match value {
            ReserveDegradedReason::NoFeasibleRescue => Self::NoFeasibleRescue,
            ReserveDegradedReason::ProtectionFailed => Self::ProtectionFailed,
        }
    }
}

impl From<RecordedReserveDegradedReason> for ReserveDegradedReason {
    fn from(value: RecordedReserveDegradedReason) -> Self {
        match value {
            RecordedReserveDegradedReason::NoFeasibleRescue => Self::NoFeasibleRescue,
            RecordedReserveDegradedReason::ProtectionFailed => Self::ProtectionFailed,
        }
    }
}

const fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

const fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}
