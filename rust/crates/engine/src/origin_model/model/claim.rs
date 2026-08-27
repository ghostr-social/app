use super::{circuit_key, OriginModel, EXPLORATION_SAMPLES, SPARSE_PROBE_BYTES};
use crate::origin_model::circuit::{CircuitStatus, RecoveryClaim, RecoveryStage};
use crate::origin_model::exploration::ExplorationClaim;
use crate::origin_model::{DecisionMode, OriginObservation, OriginQuery};

mod completion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    Production,
    Exploration { maximum_bytes: u64 },
    RecoveryProbe { maximum_bytes: u64 },
    RecoveryTrial,
    Blocked,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AdmissionClaim(ClaimKind);

#[derive(Clone, Copy, Debug)]
pub enum AdmissionClaimTerminal<'a> {
    NotStarted,
    StartedWithoutObservation,
    Observed(&'a OriginObservation),
    ObservedWholeBody(&'a OriginObservation),
}

#[derive(Debug, Eq, PartialEq)]
enum ClaimKind {
    Exploration(ExplorationClaim),
    Recovery(RecoveryClaim),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClaimedAdmission {
    admission: Admission,
    claim: Option<AdmissionClaim>,
}

impl ClaimedAdmission {
    pub const fn admission(&self) -> Admission {
        self.admission
    }

    pub fn into_parts(self) -> (Admission, Option<AdmissionClaim>) {
        (self.admission, self.claim)
    }

    const fn without_claim(admission: Admission) -> Self {
        Self {
            admission,
            claim: None,
        }
    }

    fn with_claim(admission: Admission, claim: ClaimKind) -> Self {
        Self {
            admission,
            claim: Some(AdmissionClaim(claim)),
        }
    }
}

impl OriginModel {
    pub fn claim(&mut self, query: &OriginQuery, now: u64, mode: DecisionMode) -> ClaimedAdmission {
        let key = circuit_key(query);
        match self.circuits.status(&key, now) {
            CircuitStatus::Open => return blocked(),
            CircuitStatus::RecoveryProbe | CircuitStatus::RecoveryTrial => {
                return self.claim_recovery(key, now);
            }
            CircuitStatus::Closed => {}
        }
        let samples = self.estimate(query, now, mode).effective_samples;
        if mode != DecisionMode::Normal || samples >= EXPLORATION_SAMPLES {
            return ClaimedAdmission::without_claim(Admission::Production);
        }
        self.claim_exploration(query, now)
    }

    pub fn circuit_admission(&self, query: &OriginQuery, now: u64) -> Admission {
        match self.circuits.status(&circuit_key(query), now) {
            CircuitStatus::Closed => Admission::Production,
            CircuitStatus::Open => Admission::Blocked,
            CircuitStatus::RecoveryProbe => Admission::RecoveryProbe {
                maximum_bytes: SPARSE_PROBE_BYTES,
            },
            CircuitStatus::RecoveryTrial => Admission::RecoveryTrial,
        }
    }

    fn claim_recovery(
        &mut self,
        key: crate::origin_model::keys::OriginMethodKey,
        now: u64,
    ) -> ClaimedAdmission {
        let Some(claim) = self.circuits.claim(key, now) else {
            return blocked();
        };
        let admission = match claim.stage() {
            RecoveryStage::Probe => Admission::RecoveryProbe {
                maximum_bytes: SPARSE_PROBE_BYTES,
            },
            RecoveryStage::Trial => Admission::RecoveryTrial,
        };
        ClaimedAdmission::with_claim(admission, ClaimKind::Recovery(claim))
    }

    fn claim_exploration(&mut self, query: &OriginQuery, now: u64) -> ClaimedAdmission {
        let Some(claim) = self.exploration.claim(query.origin(), now) else {
            return blocked();
        };
        ClaimedAdmission::with_claim(
            Admission::Exploration {
                maximum_bytes: SPARSE_PROBE_BYTES,
            },
            ClaimKind::Exploration(claim),
        )
    }
}

const fn blocked() -> ClaimedAdmission {
    ClaimedAdmission::without_claim(Admission::Blocked)
}
