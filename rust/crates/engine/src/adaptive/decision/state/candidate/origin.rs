use super::super::super::privacy::DecisionPrivacy;
use crate::adaptive::OriginHealth;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct OriginState {
    source: String,
    available: bool,
    throughput_bps: u64,
    rtt_ms: u64,
    packet_loss_bps: u16,
    failure_bps: u16,
}

impl OriginState {
    pub(super) fn capture(value: &OriginHealth, privacy: &DecisionPrivacy) -> Self {
        Self {
            source: privacy.source(&value.source),
            available: value.available,
            throughput_bps: value.throughput_bps,
            rtt_ms: value.rtt_ms,
            packet_loss_bps: value.packet_loss_bps,
            failure_bps: value.failure_bps,
        }
    }

    pub(super) fn origin(&self) -> OriginHealth {
        OriginHealth {
            source: self.source.clone(),
            available: self.available,
            throughput_bps: self.throughput_bps,
            rtt_ms: self.rtt_ms,
            packet_loss_bps: self.packet_loss_bps,
            failure_bps: self.failure_bps,
        }
    }
}
