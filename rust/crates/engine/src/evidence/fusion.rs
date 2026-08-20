use super::{Confidence, Evidence, EvidenceField, EvidenceScope, EvidenceValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod axes;
mod size;

const NETWORK_HALF_LIFE_MS: u64 = 6 * 60 * 60 * 1_000;
const STALE_BPS: u16 = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceAxes {
    pub readiness: Confidence,
    pub integrity: Confidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SizeAssessment {
    pub lower: Option<u64>,
    pub upper: Option<u64>,
    pub exact: Option<u64>,
    pub confidence: Confidence,
    pub conflict: bool,
    pub reliable: bool,
    pub resolved_by_direct_evidence: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAssessment {
    pub size: SizeAssessment,
    pub confidence: ConfidenceAxes,
    pub missing: Vec<EvidenceField>,
    #[serde(default)]
    pub stale: Vec<EvidenceField>,
    pub conflicts: Vec<EvidenceField>,
    fields: BTreeMap<EvidenceField, EvidenceValue>,
}

impl Default for EvidenceAssessment {
    fn default() -> Self {
        Self {
            size: SizeAssessment {
                lower: None,
                upper: None,
                exact: None,
                confidence: Confidence::none(),
                conflict: false,
                reliable: false,
                resolved_by_direct_evidence: false,
            },
            confidence: ConfidenceAxes {
                readiness: Confidence::none(),
                integrity: Confidence::none(),
            },
            missing: EvidenceField::ALL.to_vec(),
            stale: Vec::new(),
            conflicts: Vec::new(),
            fields: BTreeMap::new(),
        }
    }
}

impl EvidenceAssessment {
    pub fn value(&self, field: EvidenceField) -> Option<&EvidenceValue> {
        self.fields.get(&field)
    }
}

pub(crate) fn assess(
    records: &[Evidence<EvidenceValue>],
    url: &str,
    now_ms: u64,
) -> EvidenceAssessment {
    let active: Vec<_> = records.iter().filter(|item| applies(item, url)).collect();
    let mut fields = BTreeMap::new();
    let mut missing = Vec::new();
    let mut stale = Vec::new();
    let mut conflicts = Vec::new();
    for field in EvidenceField::ALL {
        let observed = matching(&active, field);
        if observed.is_empty() {
            missing.push(field);
            continue;
        }
        let matching = fresh(&observed, now_ms);
        if matching.is_empty() {
            stale.push(field);
            continue;
        }
        if has_conflict(&matching) {
            conflicts.push(field);
        }
        if let Some(winner) = winner(&matching, now_ms) {
            fields.insert(field, winner.value.clone());
        }
    }
    EvidenceAssessment {
        size: size::assess(&active, now_ms),
        confidence: axes::assess(&active, now_ms),
        missing,
        stale,
        conflicts,
        fields,
    }
}

fn applies(item: &Evidence<EvidenceValue>, url: &str) -> bool {
    item.is_valid() && item.scope.url_value().is_none_or(|value| value == url)
}

pub(super) fn matching<'a>(
    records: &'a [&Evidence<EvidenceValue>],
    field: EvidenceField,
) -> Vec<&'a Evidence<EvidenceValue>> {
    records
        .iter()
        .copied()
        .filter(|item| item.value.field() == field)
        .collect()
}

pub(super) fn winner<'a>(
    records: &[&'a Evidence<EvidenceValue>],
    now_ms: u64,
) -> Option<&'a Evidence<EvidenceValue>> {
    records
        .iter()
        .copied()
        .filter(|item| is_fresh(item, now_ms))
        .max_by_key(|item| {
            (
                item.source.priority(),
                effective_confidence(item, now_ms),
                item.observed_at_ms,
            )
        })
}

pub(super) fn agreement(
    winner: &Evidence<EvidenceValue>,
    records: &[&Evidence<EvidenceValue>],
    now_ms: u64,
) -> Confidence {
    let count = records
        .iter()
        .filter(|item| item.value == winner.value && is_fresh(item, now_ms))
        .count();
    effective_confidence(winner, now_ms).with_agreement(count)
}

pub(super) fn effective_confidence(item: &Evidence<EvidenceValue>, now_ms: u64) -> Confidence {
    let stable = item.observed_at_ms == 0
        || item.source.structural() && matches!(item.scope, EvidenceScope::ImmutableBytes(_))
        || item
            .validator
            .as_ref()
            .is_some_and(|value| value.is_strong());
    match stable {
        true => item.confidence,
        false => item.confidence.decayed(
            now_ms.saturating_sub(item.observed_at_ms),
            NETWORK_HALF_LIFE_MS,
        ),
    }
}

pub(super) fn fresh<'a>(
    records: &[&'a Evidence<EvidenceValue>],
    now_ms: u64,
) -> Vec<&'a Evidence<EvidenceValue>> {
    records
        .iter()
        .copied()
        .filter(|item| is_fresh(item, now_ms))
        .collect()
}

fn is_fresh(item: &Evidence<EvidenceValue>, now_ms: u64) -> bool {
    effective_confidence(item, now_ms).basis_points() >= STALE_BPS
}

fn has_conflict(records: &[&Evidence<EvidenceValue>]) -> bool {
    distinct_count(records) > 1
}

pub(super) fn distinct_count(records: &[&Evidence<EvidenceValue>]) -> usize {
    let mut distinct: Vec<&EvidenceValue> = Vec::new();
    for item in records {
        if !distinct.contains(&&item.value) {
            distinct.push(&item.value);
        }
    }
    distinct.len()
}
