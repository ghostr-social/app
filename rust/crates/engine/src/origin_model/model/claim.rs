use super::{circuit_key, OriginModel, EXPLORATION_SAMPLES, SPARSE_PROBE_BYTES};
use crate::origin_model::circuit::{CircuitStatus, RecoveryClaim, RecoveryStage};
use crate::origin_model::exploration::ExplorationClaim;
use crate::origin_model::{DecisionMode, OriginAdmissionIntent, OriginObservation, OriginQuery};

mod completion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    Production,
    Exploration,
    RecoveryProbe { maximum_bytes: u64 },
    RecoveryTrial,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionBlockReason {
    CircuitOpen,
    RecoveryLease,
    ExplorationBudgetExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionPath {
    Production,
    Exploration,
    Recovery,
    Blocked(AdmissionBlockReason),
}

#[derive(Debug, Eq, PartialEq)]
pub struct AdmissionClaim(ClaimKind);

impl AdmissionClaim {
    pub const fn is_exploration(&self) -> bool {
        matches!(self.0, ClaimKind::Exploration(_))
    }
}

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
    block_reason: Option<AdmissionBlockReason>,
}

impl ClaimedAdmission {
    pub const fn block_reason(&self) -> Option<AdmissionBlockReason> {
        self.block_reason
    }

    pub fn into_parts(self) -> (Admission, Option<AdmissionClaim>) {
        (self.admission, self.claim)
    }

    const fn without_claim(admission: Admission) -> Self {
        Self {
            admission,
            claim: None,
            block_reason: None,
        }
    }

    fn with_claim(admission: Admission, claim: ClaimKind) -> Self {
        Self {
            admission,
            claim: Some(AdmissionClaim(claim)),
            block_reason: None,
        }
    }
}

impl OriginModel {
    pub fn claim(
        &mut self,
        query: &OriginQuery,
        now: u64,
        mode: DecisionMode,
        intent: OriginAdmissionIntent,
    ) -> ClaimedAdmission {
        match self.admission_path(query, now, mode, intent) {
            AdmissionPath::Production => ClaimedAdmission::without_claim(Admission::Production),
            AdmissionPath::Exploration => self.claim_exploration(query, now),
            AdmissionPath::Recovery => self.claim_recovery(circuit_key(query), now),
            AdmissionPath::Blocked(reason) => blocked(reason),
        }
    }

    pub(crate) fn admission_block_reason(
        &self,
        query: &OriginQuery,
        now: u64,
        mode: DecisionMode,
        intent: OriginAdmissionIntent,
    ) -> Option<AdmissionBlockReason> {
        match self.admission_path(query, now, mode, intent) {
            AdmissionPath::Blocked(reason) => Some(reason),
            _ => None,
        }
    }

    pub fn circuit_admission(&self, query: &OriginQuery, now: u64) -> Admission {
        match self.circuits.status(&circuit_key(query), now) {
            CircuitStatus::Closed => Admission::Production,
            CircuitStatus::Open | CircuitStatus::RecoveryLease => Admission::Blocked,
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
            return blocked(AdmissionBlockReason::RecoveryLease);
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
            return blocked(AdmissionBlockReason::ExplorationBudgetExhausted);
        };
        ClaimedAdmission::with_claim(Admission::Exploration, ClaimKind::Exploration(claim))
    }

    fn admission_path(
        &self,
        query: &OriginQuery,
        now: u64,
        mode: DecisionMode,
        intent: OriginAdmissionIntent,
    ) -> AdmissionPath {
        match self.circuits.status(&circuit_key(query), now) {
            CircuitStatus::Open => AdmissionPath::Blocked(AdmissionBlockReason::CircuitOpen),
            CircuitStatus::RecoveryLease => {
                AdmissionPath::Blocked(AdmissionBlockReason::RecoveryLease)
            }
            CircuitStatus::RecoveryProbe | CircuitStatus::RecoveryTrial => AdmissionPath::Recovery,
            CircuitStatus::Closed => self.closed_path(query, now, mode, intent),
        }
    }

    fn closed_path(
        &self,
        query: &OriginQuery,
        now: u64,
        mode: DecisionMode,
        intent: OriginAdmissionIntent,
    ) -> AdmissionPath {
        if intent == OriginAdmissionIntent::Delivery {
            return AdmissionPath::Production;
        }
        let samples = self.estimate(query, now, mode).effective_samples;
        if mode != DecisionMode::Normal || samples >= EXPLORATION_SAMPLES {
            return AdmissionPath::Production;
        }
        if self.exploration.can_claim(query.origin(), now) {
            return AdmissionPath::Exploration;
        }
        AdmissionPath::Blocked(AdmissionBlockReason::ExplorationBudgetExhausted)
    }
}

const fn blocked(reason: AdmissionBlockReason) -> ClaimedAdmission {
    ClaimedAdmission {
        admission: Admission::Blocked,
        claim: None,
        block_reason: Some(reason),
    }
}

#[cfg(any(test, feature = "test"))]
mod test_support;
