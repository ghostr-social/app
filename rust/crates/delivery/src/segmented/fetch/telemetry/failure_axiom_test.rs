use super::*;

impl FetchFailure {
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
}
