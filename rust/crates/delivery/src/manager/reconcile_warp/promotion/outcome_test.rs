use ghostr_engine::adaptive::DecisionOutcome;

#[test]
fn promotion_outcome_preserves_measured_transaction_time() {
    assert_eq!(
        super::outcome(Ok(()), 7),
        DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 7,
        }
    );
    assert_eq!(
        super::outcome(Err("rejected"), 9),
        DecisionOutcome::Failed {
            class: "rejected".into(),
            elapsed_ms: 9,
        }
    );
}
