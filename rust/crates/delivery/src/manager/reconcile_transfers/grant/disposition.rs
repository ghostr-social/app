use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::origin_model::AdmissionBlockReason;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GrantRejection {
    Duplicate,
    RetryCooling,
    StorePressure,
    Origin(AdmissionBlockReason),
    InvalidAuthority,
    AdmissionInvariant,
}

pub(super) fn outcome_for_rejection(rejection: GrantRejection) -> DecisionOutcome {
    match rejection {
        GrantRejection::Duplicate | GrantRejection::Origin(AdmissionBlockReason::RecoveryLease) => {
            DecisionOutcome::Superseded
        }
        GrantRejection::Origin(AdmissionBlockReason::CircuitOpen) => {
            failed("warp_origin_circuit_open")
        }
        GrantRejection::Origin(AdmissionBlockReason::ExplorationBudgetExhausted) => {
            failed("warp_origin_exploration_budget_exhausted")
        }
        GrantRejection::InvalidAuthority => failed("warp_origin_authority_invalid"),
        GrantRejection::RetryCooling => failed("warp_retry_cooling"),
        GrantRejection::StorePressure => failed("warp_store_pressure_parked"),
        GrantRejection::AdmissionInvariant => failed("warp_origin_admission_inconsistent"),
    }
}

fn failed(class: &str) -> DecisionOutcome {
    DecisionOutcome::Failed {
        class: class.into(),
        elapsed_ms: 0,
    }
}
