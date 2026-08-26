use super::*;
use crate::evidence::SizeAssessment;

impl CatalogEntry {
    pub fn conservative_size_for(&self, source: &str, now_ms: u64) -> SizeAssessment {
        self.evidence_assessment_for(source, now_ms).size
    }

    pub fn current_validator_for(
        &self,
        source: &str,
    ) -> Option<&crate::evidence::EvidenceValidator> {
        self.ledger.current_validator(source)
    }
}
