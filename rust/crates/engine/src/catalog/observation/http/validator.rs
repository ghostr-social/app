use crate::catalog::CatalogEntry;

impl CatalogEntry {
    pub(super) fn apply_invalidation(
        &mut self,
        invalidation: crate::evidence::EvidenceInvalidation,
    ) {
        if invalidation.structural_evidence {
            self.timeline = None;
            self.tail_timeline_needed = false;
        }
    }
}
