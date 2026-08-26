use super::{
    PlayerPreparationAttempt, PlayerPreparationClaim, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use ghostr_engine::PostId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreparationFollowup {
    claim: PlayerPreparationClaim,
    attempt: PlayerPreparationAttempt,
    sequence: u64,
    observation: PlayerPreparationObservation,
}

impl PlayerPreparationFollowup {
    pub fn try_new(
        claim: PlayerPreparationClaim,
        attempt: PlayerPreparationAttempt,
        sequence: u64,
        observation: PlayerPreparationObservation,
    ) -> Option<Self> {
        (sequence > 0).then_some(Self {
            claim,
            attempt,
            sequence,
            observation,
        })
    }

    pub(crate) fn post(&self) -> &PostId {
        self.claim.post()
    }

    pub(crate) fn client_epoch(&self) -> u64 {
        self.attempt.client_epoch
    }

    pub(crate) fn player_capability_generation(&self) -> u64 {
        self.attempt.player_capability_generation
    }

    pub(crate) fn attempt_generation(&self) -> u64 {
        self.attempt.attempt_generation
    }

    fn sequence(&self) -> u64 {
        self.sequence
    }

    fn state(&self) -> PlayerPreparationState {
        self.observation.state
    }

    pub(crate) fn is_terminal(&self) -> bool {
        matches!(
            self.state(),
            PlayerPreparationState::Failed | PlayerPreparationState::Released
        )
    }

    pub(crate) fn same_receipt_key(&self, report: &PlayerPreparationReport) -> bool {
        self.post() == report.post()
            && self.player_capability_generation() == report.player_capability_generation()
            && self.client_epoch() == report.client_epoch()
            && self.attempt_generation() == report.attempt_generation()
            && self.sequence() == report.sequence()
    }

    pub(crate) fn matches_report(&self, report: &PlayerPreparationReport) -> bool {
        self.same_receipt_key(report)
            && self.claim == PlayerPreparationClaim::from_authority(&report.authority)
            && self.observation == report.observation
    }

    pub(crate) fn same_attempt(&self, report: &PlayerPreparationReport) -> bool {
        self.player_capability_generation() == report.player_capability_generation()
            && self.client_epoch() == report.client_epoch()
            && self.attempt_generation() == report.attempt_generation()
    }

    pub(crate) fn anchor_to(
        self,
        admitted: &PlayerPreparationReport,
    ) -> Option<PlayerPreparationReport> {
        if self.attempt != admitted.attempt || !admitted.authority.accepts(&self.claim) {
            return None;
        }
        PlayerPreparationReport::try_new(
            admitted.authority.clone(),
            self.attempt,
            self.sequence,
            self.observation,
        )
    }
}

#[cfg(test)]
#[path = "followup/test_support.rs"]
mod test_support;
