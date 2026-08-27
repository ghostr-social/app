use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{Admission, AdmissionClaimTerminal, RequestMethod};

#[test]
fn started_attempt_without_observation_releases_recovery_neutrally() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = open_circuit(&full);
    let expected = Admission::RecoveryProbe {
        maximum_bytes: 65_536,
    };
    let active = claim(&mut model, &full, 5_000, expected);

    assert!(model.complete_claim(active, AdmissionClaimTerminal::StartedWithoutObservation));

    claim(&mut model, &full, 5_001, expected);
}
