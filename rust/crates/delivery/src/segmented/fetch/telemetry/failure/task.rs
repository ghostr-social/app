use super::{FailureEvidence, FetchFailure};
use crate::segmented::fetch::failure_policy::FailurePolicy;
use crate::segmented::fetch::telemetry::FetchProgress;
use ghostr_engine::origin_model::ErrorReason;

#[derive(Clone, Copy)]
pub(super) enum TaskFailure {
    Panicked,
    Cancelled,
}

impl FetchFailure {
    pub(in crate::segmented) fn task_failed(
        error: tokio::task::JoinError,
        progress: &FetchProgress,
    ) -> Self {
        let task_failure = match error.is_panic() {
            true => TaskFailure::Panicked,
            false => TaskFailure::Cancelled,
        };
        let mut failure = Self::failure(
            anyhow::anyhow!("HLS fetch task failed: {error}"),
            ErrorReason::Unknown,
            FailureEvidence {
                origin: progress.origin(),
                network_bytes: progress.network_bytes(),
            },
            FailurePolicy::terminal(),
        );
        failure.task_failure = Some(task_failure);
        failure.response_completed = progress.response_completed();
        failure
    }

    #[cfg(test)]
    pub(in crate::segmented) fn is_local_terminal(&self) -> bool {
        matches!(
            self.disposition(),
            crate::segmented::fetch::FailureDisposition::Terminal
        )
    }

    pub(in crate::segmented) fn task_failure_class(&self) -> Option<&'static str> {
        self.task_failure.map(TaskFailure::class)
    }
}

impl TaskFailure {
    const fn class(self) -> &'static str {
        match self {
            Self::Panicked => "warp_hls_task_panicked",
            Self::Cancelled => "warp_hls_task_cancelled",
        }
    }
}
