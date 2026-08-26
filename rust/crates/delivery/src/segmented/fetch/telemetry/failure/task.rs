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
        let task_failure = if error.is_panic() {
            TaskFailure::Panicked
        } else {
            TaskFailure::Cancelled
        };
        let error = anyhow::Error::new(error).context("HLS fetch task failed");
        let mut failure = Self::failure(
            error,
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

#[cfg(test)]
#[path = "task_axiom_test.rs"]
pub(crate) mod axiom_test_support;
