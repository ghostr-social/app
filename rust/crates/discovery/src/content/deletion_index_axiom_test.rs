use super::*;

impl DeletionIndex {
    pub(crate) fn with_retention(retention: usize) -> Self {
        Self {
            pending: PendingDeletions::with_retention(retention),
            ..Self::default()
        }
    }
    pub(crate) fn retained_claims(&self) -> usize {
        self.anchored
            .values()
            .filter(|claim| claim.is_some())
            .count()
            + self.pending.len()
    }
}
