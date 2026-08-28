use super::disposition::{outcome_for_rejection, GrantRejection};
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::origin_model::AdmissionBlockReason;

#[test]
fn selected_grant_rejections_have_truthful_terminal_outcomes() {
    let superseded = [
        GrantRejection::Duplicate,
        GrantRejection::Origin(AdmissionBlockReason::RecoveryLease),
    ];
    for rejection in superseded {
        assert_eq!(
            outcome_for_rejection(rejection),
            DecisionOutcome::Superseded
        );
    }
    assert_failed(
        GrantRejection::Origin(AdmissionBlockReason::CircuitOpen),
        "warp_origin_circuit_open",
    );
    assert_failed(
        GrantRejection::Origin(AdmissionBlockReason::ExplorationBudgetExhausted),
        "warp_origin_exploration_budget_exhausted",
    );
    assert_failed(
        GrantRejection::InvalidAuthority,
        "warp_origin_authority_invalid",
    );
    assert_failed(GrantRejection::RetryCooling, "warp_retry_cooling");
    assert_failed(GrantRejection::StorePressure, "warp_store_pressure_parked");
    assert_failed(
        GrantRejection::AdmissionInvariant,
        "warp_origin_admission_inconsistent",
    );
}

fn assert_failed(rejection: GrantRejection, expected: &str) {
    let DecisionOutcome::Failed { class, elapsed_ms } = outcome_for_rejection(rejection) else {
        panic!("expected failed outcome");
    };
    assert_eq!(class, expected);
    assert_eq!(elapsed_ms, 0);
}
