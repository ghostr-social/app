use super::{FetchProblem, FetchProgress, OriginTelemetry};
use crate::segmented::fetch::failure_policy::FailurePolicy;
use ghostr_engine::origin_model::ErrorReason;
use reqwest::StatusCode;

mod evidence;
mod task;
use task::TaskFailure;

pub(in crate::segmented) struct FetchFailure {
    error: anyhow::Error,
    reason: ErrorReason,
    origin: Option<OriginTelemetry>,
    admitted: bool,
    network_bytes: u64,
    cancelled: bool,
    superseded: bool,
    status: Option<StatusCode>,
    policy: FailurePolicy,
    task_failure: Option<TaskFailure>,
}

struct FailureEvidence {
    origin: Option<OriginTelemetry>,
    network_bytes: u64,
}

impl FetchFailure {
    pub(in crate::segmented::fetch) fn new(
        problem: FetchProblem,
        progress: &FetchProgress,
    ) -> Self {
        Self {
            error: problem.error,
            reason: problem.reason,
            origin: progress.origin(),
            admitted: progress.has_admission(),
            network_bytes: progress.network_bytes(),
            cancelled: false,
            superseded: false,
            status: problem.status,
            policy: problem.policy,
            task_failure: None,
        }
    }

    pub(in crate::segmented) fn preflight(error: anyhow::Error, reason: ErrorReason) -> Self {
        Self::failure(
            error,
            reason,
            FailureEvidence {
                origin: None,
                network_bytes: 0,
            },
            FailurePolicy::neutral(),
        )
    }

    pub(in crate::segmented) fn admitted(
        error: anyhow::Error,
        reason: ErrorReason,
        origin: OriginTelemetry,
        network_bytes: u64,
    ) -> Self {
        Self::failure(
            error,
            reason,
            FailureEvidence {
                origin: Some(origin),
                network_bytes,
            },
            FailurePolicy::for_reason(reason),
        )
    }

    #[cfg(test)]
    pub(in crate::segmented) fn admitted_neutral(
        error: anyhow::Error,
        reason: ErrorReason,
        origin: OriginTelemetry,
        network_bytes: u64,
    ) -> Self {
        Self::failure(
            error,
            reason,
            FailureEvidence {
                origin: Some(origin),
                network_bytes,
            },
            FailurePolicy::neutral(),
        )
    }

    pub(in crate::segmented) fn cancelled(
        origin: Option<OriginTelemetry>,
        network_bytes: u64,
    ) -> Self {
        let mut failure = Self::failure(
            anyhow::anyhow!("HLS bootstrap cancelled"),
            ErrorReason::Policy,
            FailureEvidence {
                origin,
                network_bytes,
            },
            FailurePolicy::neutral(),
        );
        failure.cancelled = true;
        failure
    }

    pub(in crate::segmented) fn superseded(origin: OriginTelemetry, network_bytes: u64) -> Self {
        let mut failure = Self::failure(
            anyhow::anyhow!("HLS bootstrap publication superseded"),
            ErrorReason::Policy,
            FailureEvidence {
                origin: Some(origin),
                network_bytes,
            },
            FailurePolicy::neutral(),
        );
        failure.superseded = true;
        failure
    }

    fn failure(
        error: anyhow::Error,
        reason: ErrorReason,
        evidence: FailureEvidence,
        policy: FailurePolicy,
    ) -> Self {
        Self {
            error,
            reason,
            origin: evidence.origin,
            admitted: evidence.origin.is_some(),
            network_bytes: evidence.network_bytes,
            cancelled: false,
            superseded: false,
            status: None,
            policy,
            task_failure: None,
        }
    }
}
