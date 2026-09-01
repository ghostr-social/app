use super::{TransportRescue, TransportRescueReason};

/// Bounded additive feedback for rescue focuses removed by mailbox coalescing.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TransportRescueFeedback {
    substitutions: u64,
    rank_displacement_total: u64,
    wait_total_ms: u64,
    reason_counts: [u64; 4],
}

impl TransportRescueFeedback {
    pub(crate) fn record(&mut self, rescue: TransportRescue) {
        self.substitutions = self.substitutions.saturating_add(1);
        self.rank_displacement_total = self
            .rank_displacement_total
            .saturating_add(u64::from(rescue.rank_displacement));
        self.wait_total_ms = self.wait_total_ms.saturating_add(rescue.wait_ms);
        let count = &mut self.reason_counts[reason_index(rescue.reason)];
        *count = count.saturating_add(1);
    }

    pub(crate) fn substitutions(self) -> u64 {
        self.substitutions
    }

    pub(crate) fn rank_displacement_total(self) -> u64 {
        self.rank_displacement_total
    }

    pub(crate) fn wait_total_ms(self) -> u64 {
        self.wait_total_ms
    }

    pub(crate) fn reason_count(self, reason: TransportRescueReason) -> u64 {
        self.reason_counts[reason_index(reason)]
    }
}

const fn reason_index(reason: TransportRescueReason) -> usize {
    match reason {
        TransportRescueReason::EtaUnavailable => 0,
        TransportRescueReason::EtaTooLong => 1,
        TransportRescueReason::DeliveryFailed => 2,
        TransportRescueReason::GraceExpired => 3,
    }
}
