use super::*;

impl FetchFailure {
    pub(in crate::segmented) fn is_local_terminal(&self) -> bool {
        matches!(
            self.disposition(),
            crate::segmented::fetch::FailureDisposition::Terminal
        )
    }
}
