use super::{CircuitBreaker, OriginMethodKey, PROBE_LEASE_MS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CircuitStatus {
    Closed,
    Open,
    RecoveryProbe,
    RecoveryTrial,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RecoveryClaim {
    pub(super) key: OriginMethodKey,
    pub(super) generation: u64,
    pub(super) stage: RecoveryStage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryStage {
    Probe,
    Trial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryResolution {
    Recovered { at_ms: u64 },
    TrialRequired { at_ms: u64 },
    Deferred { at_ms: u64 },
    Failed { at_ms: u64 },
    Released,
}

impl CircuitBreaker {
    pub(super) fn claim_recovery(&mut self, at_ms: u64) -> Option<(u64, RecoveryStage)> {
        let stage = match self.status(at_ms) {
            CircuitStatus::RecoveryProbe => RecoveryStage::Probe,
            CircuitStatus::RecoveryTrial => RecoveryStage::Trial,
            CircuitStatus::Closed | CircuitStatus::Open => return None,
        };
        self.probe_generation = self.probe_generation.checked_add(1)?;
        self.probe_lease_until_ms = at_ms.saturating_add(PROBE_LEASE_MS);
        Some((self.probe_generation, stage))
    }

    pub(super) fn complete_recovery(
        &mut self,
        claim: &RecoveryClaim,
        resolution: RecoveryResolution,
    ) -> bool {
        if !self.matches(claim) {
            return false;
        }
        match resolution {
            RecoveryResolution::Recovered { at_ms } => self.observe(true, at_ms),
            RecoveryResolution::TrialRequired { at_ms } => {
                if claim.stage != RecoveryStage::Probe {
                    return false;
                }
                self.require_trial(at_ms);
            }
            RecoveryResolution::Deferred { at_ms } => self.defer_recovery(at_ms),
            RecoveryResolution::Failed { at_ms } => self.observe(false, at_ms),
            RecoveryResolution::Released => self.probe_lease_until_ms = 0,
        }
        true
    }

    pub(super) fn matches(&self, claim: &RecoveryClaim) -> bool {
        let stage_matches = self.trial_pending == (claim.stage == RecoveryStage::Trial);
        self.probe_generation == claim.generation && self.probe_lease_until_ms != 0 && stage_matches
    }

    fn require_trial(&mut self, at_ms: u64) {
        self.last_at_ms = at_ms;
        self.probe_lease_until_ms = 0;
        self.trial_pending = true;
    }
}

impl RecoveryClaim {
    pub(crate) fn key(&self) -> &OriginMethodKey {
        &self.key
    }

    pub(crate) const fn stage(&self) -> RecoveryStage {
        self.stage
    }
}
