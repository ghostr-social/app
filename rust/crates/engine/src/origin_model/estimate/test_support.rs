use super::*;

impl OriginEstimate {
    pub(crate) fn most_likely_error(&self) -> Option<ErrorReason> {
        self.error_frequencies
            .iter()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(reason, _)| *reason)
    }
}
