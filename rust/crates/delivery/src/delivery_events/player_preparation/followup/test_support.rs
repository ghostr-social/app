use super::*;

impl PlayerPreparationFollowup {
    pub(crate) fn from_report(report: PlayerPreparationReport) -> Self {
        Self {
            claim: PlayerPreparationClaim::from_authority(&report.authority),
            attempt: report.attempt,
            sequence: report.sequence,
            observation: report.observation,
        }
    }
}
