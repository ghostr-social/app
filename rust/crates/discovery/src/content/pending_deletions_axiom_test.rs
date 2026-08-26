use super::*;

impl PendingDeletions {
    pub(in super::super) fn len(&self) -> usize {
        self.claims.len()
    }
    pub(in super::super) fn with_retention(retention: usize) -> Self {
        Self {
            retention: retention.max(1),
            ..Self::default()
        }
    }
}
