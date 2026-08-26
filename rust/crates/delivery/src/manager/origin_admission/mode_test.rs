use super::mode;
use crate::tests::support::planned_transfer;
use ghostr_engine::adaptive::{ControlMode, PreemptionAuthority};
use ghostr_engine::origin_model::DecisionMode;

#[test]
fn speculative_preemption_does_not_demote_safety_origin_admission() {
    let mut transfer = planned_transfer("far-reserve", "same", PreemptionAuthority::Speculative);
    transfer.control_mode = ControlMode::Safety;

    assert_eq!(mode(&transfer), DecisionMode::Safety);
}
