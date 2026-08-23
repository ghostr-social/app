use super::{
    PlayerPreparationAttempt, PlayerPreparationClaim, PlayerPreparationObservation,
    PlayerPreparationReport,
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

    pub(crate) fn from_report(report: PlayerPreparationReport) -> Self {
        Self {
            claim: PlayerPreparationClaim::from_authority(&report.authority),
            attempt: report.attempt,
            sequence: report.sequence,
            observation: report.observation,
        }
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
