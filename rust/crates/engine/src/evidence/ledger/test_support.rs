use super::*;

impl EvidenceInvalidation {
    pub(crate) fn is_empty(self) -> bool {
        self.invalidated_records == 0
    }
}
