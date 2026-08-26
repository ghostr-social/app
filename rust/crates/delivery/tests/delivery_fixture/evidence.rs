use ghostr_delivery::delivery_events::{DecisionHistorySnapshot, DeliveryHandle};
use ghostr_delivery::evaluation::EvaluationSnapshot;
use serde::Deserialize;

pub trait DeliveryEvidence {
    fn decision_history(&self) -> DecisionHistorySnapshot;
    fn evaluation_snapshot(&self) -> EvaluationSnapshot;
}

impl DeliveryEvidence for DeliveryHandle {
    fn decision_history(&self) -> DecisionHistorySnapshot {
        let json = self
            .decision_history_json()
            .expect("serializable decision evidence");
        serde_json::from_str::<DecisionEnvelope>(&json)
            .expect("valid decision evidence schema")
            .decisions
    }

    fn evaluation_snapshot(&self) -> EvaluationSnapshot {
        let json = self
            .evidence_page_json(0, 0)
            .expect("serializable delivery evidence");
        serde_json::from_str::<DeliveryEnvelope>(&json)
            .expect("valid delivery evidence schema")
            .evaluation
    }
}

#[derive(Deserialize)]
struct DecisionEnvelope {
    decisions: DecisionHistorySnapshot,
}

#[derive(Deserialize)]
struct DeliveryEnvelope {
    evaluation: EvaluationSnapshot,
}
