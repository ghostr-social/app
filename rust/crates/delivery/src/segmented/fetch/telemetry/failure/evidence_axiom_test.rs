use super::*;

use reqwest::StatusCode;

impl FetchFailure {
    pub(in crate::segmented) const fn status(&self) -> Option<StatusCode> {
        self.status
    }
    pub(in crate::segmented) fn retry_class(
        &self,
    ) -> Option<crate::manager::failure::FailureClass> {
        match self.disposition() {
            FailureDisposition::Retry(class) => Some(class),
            FailureDisposition::Requeue
            | FailureDisposition::RestartObject
            | FailureDisposition::Terminal => None,
        }
    }
}
