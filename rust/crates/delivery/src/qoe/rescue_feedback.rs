use super::QoeTracker;
use crate::delivery_events::{TransportRescueFeedback, TransportRescueReason};

impl QoeTracker {
    pub(crate) fn note_rescue_feedback(&mut self, feedback: TransportRescueFeedback) {
        self.stats.transport_substitutions = self
            .stats
            .transport_substitutions
            .saturating_add(feedback.substitutions());
        self.stats.rank_displacement_total = self
            .stats
            .rank_displacement_total
            .saturating_add(feedback.rank_displacement_total());
        self.stats.rescue_wait_total_ms = self
            .stats
            .rescue_wait_total_ms
            .saturating_add(feedback.wait_total_ms());
        self.add_rescue_reason_counts(feedback);
    }

    fn add_rescue_reason_counts(&mut self, feedback: TransportRescueFeedback) {
        use TransportRescueReason::{DeliveryFailed, EtaTooLong, EtaUnavailable, GraceExpired};
        let stats = &mut self.stats;
        stats.eta_unavailable_rescues = stats
            .eta_unavailable_rescues
            .saturating_add(feedback.reason_count(EtaUnavailable));
        stats.eta_too_long_rescues = stats
            .eta_too_long_rescues
            .saturating_add(feedback.reason_count(EtaTooLong));
        stats.delivery_failed_rescues = stats
            .delivery_failed_rescues
            .saturating_add(feedback.reason_count(DeliveryFailed));
        stats.grace_expired_rescues = stats
            .grace_expired_rescues
            .saturating_add(feedback.reason_count(GraceExpired));
    }
}
