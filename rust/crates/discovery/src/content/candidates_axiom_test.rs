use super::*;

impl CandidateRegistry {
    pub(crate) fn with_retention(retention: usize) -> Self {
        Self {
            retention: retention.max(1),
            ..Self::default()
        }
    }
    pub(crate) fn retained_coordinates(&self) -> usize {
        self.canonical.len()
    }
}
