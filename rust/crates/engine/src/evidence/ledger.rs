use super::fusion;
use super::{Evidence, EvidenceAssessment, EvidenceSource, EvidenceValidator, EvidenceValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const RECORD_CAPACITY: usize = 128;

mod validator;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EvidenceInvalidation {
    pub(crate) invalidated_records: usize,
    pub(crate) structural_evidence: bool,
    integrity_evidence: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceLedger {
    records: Vec<Evidence<EvidenceValue>>,
    validators: BTreeMap<String, EvidenceValidator>,
    #[serde(default)]
    validator_times: BTreeMap<String, super::EvidenceTime>,
    capacity: usize,
}

#[cfg(test)]
#[path = "ledger/test_support.rs"]
mod test_support;

impl Default for EvidenceLedger {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            validators: BTreeMap::new(),
            validator_times: BTreeMap::new(),
            capacity: RECORD_CAPACITY,
        }
    }
}

impl EvidenceLedger {
    pub(crate) fn record(&mut self, evidence: Evidence<EvidenceValue>) {
        self.make_room();
        self.records.push(evidence);
    }

    pub(crate) fn records(&self) -> &[Evidence<EvidenceValue>] {
        &self.records
    }

    pub(crate) fn records_mut(&mut self) -> &mut [Evidence<EvidenceValue>] {
        &mut self.records
    }

    pub(crate) fn assessment(&self, url: &str, now_ms: u64) -> EvidenceAssessment {
        fusion::assess(&self.records, url, now_ms)
    }

    pub(crate) fn invalidate_parser(&mut self, observed_at_ms: u64) -> EvidenceInvalidation {
        let mut result = EvidenceInvalidation::default();
        for item in &mut self.records {
            if matches!(item.source, EvidenceSource::Parser { .. }) {
                note_invalidation(&mut result, item, observed_at_ms);
            }
        }
        result
    }

    pub(crate) fn invalidate_nostr_issuer(
        &mut self,
        issuer: &str,
        observed_at_ms: u64,
    ) -> EvidenceInvalidation {
        let mut result = EvidenceInvalidation::default();
        for item in &mut self.records {
            let same = matches!(
                &item.source,
                EvidenceSource::Nostr { issuer: value, client: None } if value == issuer
            );
            if same {
                note_invalidation(&mut result, item, observed_at_ms);
            }
        }
        result
    }

    fn make_room(&mut self) {
        if self.records.len() < self.capacity {
            return;
        }
        let victim = self
            .records
            .iter()
            .position(|item| !item.is_valid())
            .unwrap_or(0);
        self.records.remove(victim);
    }
}

fn note_invalidation(
    result: &mut EvidenceInvalidation,
    item: &mut Evidence<EvidenceValue>,
    observed_at_ms: u64,
) {
    if !item.invalidate(observed_at_ms) {
        return;
    }
    result.invalidated_records += 1;
    result.structural_evidence |= item.value.field().structural();
    result.integrity_evidence |= matches!(item.value, EvidenceValue::IntegrityMatch { .. });
}
