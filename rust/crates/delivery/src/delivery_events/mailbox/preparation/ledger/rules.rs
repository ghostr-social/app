use super::{
    ingress, AttemptFence, PreparationLedger, ACTIVE_ATTEMPT_CAPACITY, ATTEMPT_FENCE_CAPACITY,
};
use crate::delivery_events::{
    PlayerPreparationDisposition, PlayerPreparationFollowup, PlayerPreparationIngress,
    PlayerPreparationReport,
};

impl PreparationLedger {
    pub(super) fn admit_receipt(
        &self,
        report: &PlayerPreparationReport,
    ) -> Result<(), PlayerPreparationIngress> {
        match self.receipts.probe_report(report) {
            Some(probe) => Err(ingress(probe)),
            None => Ok(()),
        }
    }

    pub(super) fn admit_initial_capacity(
        &self,
        report: &PlayerPreparationReport,
    ) -> Result<(), PlayerPreparationIngress> {
        let post_available =
            self.active.contains_key(report.post()) || self.active.len() < ACTIVE_ATTEMPT_CAPACITY;
        (post_available && self.can_record_fence(report))
            .then_some(())
            .ok_or(PlayerPreparationIngress::Saturated)
    }

    pub(super) fn admit_fence(
        &self,
        report: &PlayerPreparationReport,
    ) -> Result<(), PlayerPreparationIngress> {
        self.fence_allows(report)
            .then_some(())
            .ok_or(PlayerPreparationIngress::Rejected)
    }

    fn fence_allows(&self, report: &PlayerPreparationReport) -> bool {
        self.fence(report)
            .is_none_or(|fence| report.attempt_generation() > fence.max_attempt())
    }

    pub(super) fn release_replaced(
        &self,
        report: &PlayerPreparationReport,
    ) -> Result<Option<PlayerPreparationReport>, PlayerPreparationIngress> {
        let Some(known) = self.active.get(report.post()) else {
            return Ok(None);
        };
        if !report.supersedes(known) {
            return Err(PlayerPreparationIngress::Stale);
        }
        known
            .release_for_replacement(report)
            .map(Some)
            .ok_or(PlayerPreparationIngress::Rejected)
    }

    pub(super) fn admit_identity(
        &mut self,
        report: &PlayerPreparationReport,
    ) -> Result<(), PlayerPreparationIngress> {
        if report.client_epoch() < self.latest_client_epoch {
            return Err(PlayerPreparationIngress::Stale);
        }
        if report.client_epoch() > self.latest_client_epoch {
            self.active.clear();
            self.fences.clear();
            self.receipts.clear();
            self.latest_client_epoch = report.client_epoch();
            self.latest_capability_generation = Some(report.player_capability_generation());
        }
        (self.latest_capability_generation == Some(report.player_capability_generation()))
            .then_some(())
            .ok_or(PlayerPreparationIngress::Rejected)
    }

    fn can_record_fence(&self, report: &PlayerPreparationReport) -> bool {
        self.fence(report).is_some()
            || self.fences.len() < ATTEMPT_FENCE_CAPACITY
            || self.fences.iter().any(|fence| !self.fence_is_active(fence))
    }

    pub(super) fn record_fence(&mut self, report: &PlayerPreparationReport) {
        if let Some(index) = self.fence_index(report) {
            self.fences.remove(index);
        } else if self.fences.len() == ATTEMPT_FENCE_CAPACITY {
            let index = self
                .fences
                .iter()
                .position(|fence| !self.fence_is_active(fence))
                .expect("fence capacity checked");
            self.fences.remove(index);
        }
        self.fences.push_back(AttemptFence::capture(report));
    }

    pub(super) fn unanchored(
        &self,
        report: &PlayerPreparationFollowup,
    ) -> PlayerPreparationIngress {
        if report.is_terminal()
            || report.client_epoch() < self.latest_client_epoch
            || self.fences.iter().any(|fence| fence.fences(report))
        {
            return PlayerPreparationIngress::Stale;
        }
        if report.client_epoch() == self.latest_client_epoch
            && self.latest_capability_generation != Some(report.player_capability_generation())
        {
            return PlayerPreparationIngress::Rejected;
        }
        PlayerPreparationIngress::MissingInitial
    }

    pub(super) fn followup_mismatch(
        report: &PlayerPreparationFollowup,
        admitted: &PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        if report.same_attempt(admitted) {
            return PlayerPreparationIngress::Rejected;
        }
        if report.is_terminal()
            || (report.client_epoch(), report.attempt_generation())
                <= (admitted.client_epoch(), admitted.attempt_generation())
        {
            PlayerPreparationIngress::Stale
        } else {
            PlayerPreparationIngress::MissingInitial
        }
    }

    pub(super) fn reconcile_completion(
        &mut self,
        report: &PlayerPreparationReport,
        outcome: PlayerPreparationDisposition,
    ) -> PlayerPreparationDisposition {
        if matches!(
            outcome,
            PlayerPreparationDisposition::Stale | PlayerPreparationDisposition::Rejected
        ) {
            self.retire_active_attempt(report);
        }
        outcome
    }

    fn fence(&self, report: &PlayerPreparationReport) -> Option<&AttemptFence> {
        self.fences.iter().find(|fence| fence.matches(report))
    }

    fn fence_index(&self, report: &PlayerPreparationReport) -> Option<usize> {
        self.fences.iter().position(|fence| fence.matches(report))
    }

    fn fence_is_active(&self, fence: &AttemptFence) -> bool {
        self.active.values().any(|report| fence.matches(report))
    }

    fn retire_active_attempt(&mut self, report: &PlayerPreparationReport) {
        if self
            .active
            .get(report.post())
            .is_some_and(|active| active.same_attempt_identity(report))
        {
            self.active.remove(report.post());
        }
    }
}
