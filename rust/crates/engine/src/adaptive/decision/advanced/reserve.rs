use super::super::privacy::DecisionPrivacy;
use crate::adaptive::{
    RescueChanceEvidence, RescueTimingQuantile, ReserveAuthorityOccupancy, ReserveConstraint,
    ReserveDegradedReason,
};
use crate::RequestAuthority;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedWarpReserve {
    pub reserved_request_slots: u16,
    pub reserved_network_bytes: u64,
    pub degraded: bool,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub reserved_storage_bytes: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub reserved_cpu_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub global_request_width: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authority_occupancy: Vec<RecordedReserveAuthorityOccupancy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_action_ids: Vec<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<RecordedRescueChanceEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<RecordedReserveDegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedReserveAuthorityOccupancy {
    pub authority_id: String,
    pub occupied_request_slots: u64,
    pub request_width: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedRescueChanceEvidence {
    pub deadline_ms: u64,
    pub threshold_bps: u16,
    pub achieved_success_bps: u16,
    pub transport_success_bps: u16,
    pub timing_quantile: RecordedRescueTimingQuantile,
    pub timing_completion_ms: u64,
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
        authority_occupancy: value
            .authority_occupancy
            .iter()
            .map(|item| capture_occupancy(item, privacy))
            .collect(),
        protected_action_ids: value.protected_action_ids.clone(),
        chance: value.chance.map(RecordedRescueChanceEvidence::from),
        degraded_reason: value
            .degraded_reason
            .map(RecordedReserveDegradedReason::from),
    }
}

fn capture_occupancy(
    value: &ReserveAuthorityOccupancy,
    privacy: &DecisionPrivacy,
) -> RecordedReserveAuthorityOccupancy {
    RecordedReserveAuthorityOccupancy {
        authority_id: privacy.source(value.authority.as_str()),
        occupied_request_slots: u64::try_from(value.occupied_request_slots)
            .expect("request occupancy fits the schema-v2 counter"),
        request_width: value.request_width,
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
            .map(restore_occupancy)
            .collect::<Option<Vec<_>>>()?,
        protected_action_ids: value.protected_action_ids.clone(),
        chance: value.chance.map(RescueChanceEvidence::from),
        degraded: value.degraded,
        degraded_reason: value.degraded_reason.map(ReserveDegradedReason::from),
    })
}

fn restore_occupancy(
    value: &RecordedReserveAuthorityOccupancy,
) -> Option<ReserveAuthorityOccupancy> {
    Some(ReserveAuthorityOccupancy {
        authority: RequestAuthority::from_url(&value.authority_id)?,
        occupied_request_slots: usize::try_from(value.occupied_request_slots).ok()?,
        request_width: value.request_width,
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
