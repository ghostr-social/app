use super::{agreement, matching, winner, ConfidenceAxes};
use crate::evidence::{Confidence, Evidence, EvidenceField, EvidenceValue};

pub(super) fn assess(records: &[&Evidence<EvidenceValue>], now_ms: u64) -> ConfidenceAxes {
    ConfidenceAxes {
        readiness: positive(records, EvidenceField::Readiness, now_ms).max(positive(
            records,
            EvidenceField::FrontMoov,
            now_ms,
        )),
        integrity: integrity(records, now_ms),
    }
}

fn positive(records: &[&Evidence<EvidenceValue>], field: EvidenceField, now_ms: u64) -> Confidence {
    let records = matching(records, field);
    let Some(item) = winner(&records, now_ms) else {
        return Confidence::none();
    };
    match item.value {
        EvidenceValue::Ready(true) | EvidenceValue::FrontMoov(true) => {
            agreement(item, &records, now_ms)
        }
        _ => Confidence::none(),
    }
}

fn integrity(records: &[&Evidence<EvidenceValue>], now_ms: u64) -> Confidence {
    let records = matching(records, EvidenceField::Integrity);
    let Some(item) = winner(&records, now_ms) else {
        return Confidence::none();
    };
    match item.value {
        EvidenceValue::IntegrityMatch { matches: true, .. } => agreement(item, &records, now_ms),
        _ => Confidence::none(),
    }
}
