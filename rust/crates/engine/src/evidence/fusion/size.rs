use super::{agreement, distinct_count, fresh, matching, winner, SizeAssessment};
use crate::evidence::{Confidence, Evidence, EvidenceField, EvidenceValue};

const RELIABLE_BPS: u16 = 7_000;

pub(super) fn assess(records: &[&Evidence<EvidenceValue>], now_ms: u64) -> SizeAssessment {
    let observed = matching(records, EvidenceField::Size);
    let sizes = fresh(&observed, now_ms);
    let values: Vec<u64> = observed
        .iter()
        .filter_map(|item| size_value(&item.value))
        .collect();
    let selected = winner(&sizes, now_ms);
    let confidence = selected.map_or(Confidence::none(), |item| agreement(item, &sizes, now_ms));
    let conflict = distinct_count(&observed) > 1;
    let direct = selected.is_some_and(|item| item.source.direct_bytes());
    let reliable = confidence.basis_points() >= RELIABLE_BPS && (!conflict || direct);
    let chosen = selected.and_then(|item| size_value(&item.value));
    SizeAssessment {
        lower: values.iter().min().copied(),
        upper: values.iter().max().copied(),
        exact: reliable.then_some(chosen).flatten(),
        confidence,
        conflict,
        reliable,
        resolved_by_direct_evidence: conflict && direct,
    }
}

fn size_value(value: &EvidenceValue) -> Option<u64> {
    match value {
        EvidenceValue::SizeBytes(bytes) => Some(*bytes),
        _ => None,
    }
}
