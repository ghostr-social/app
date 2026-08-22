use ghostr_engine::adaptive::ResourceCost;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;
use std::fmt::{Display, Formatter};
use std::time::Duration;

mod progress;
pub(super) use progress::FetchProgress;
mod traffic;
pub(in crate::segmented) use traffic::SegmentedTraffic;

#[derive(Clone, Copy, Debug)]
pub(in crate::segmented) struct OriginTelemetry {
    pub elapsed: Duration,
    pub ttfb: Option<Duration>,
    pub concurrency: usize,
    pub network_class: NetworkClass,
}

pub(in crate::segmented) struct FetchFailure {
    error: anyhow::Error,
    reason: ErrorReason,
    origin: Option<OriginTelemetry>,
    admitted: bool,
    network_bytes: u64,
    cancelled: bool,
    superseded: bool,
}

impl FetchFailure {
    pub(super) fn new(problem: FetchProblem, progress: &FetchProgress) -> Self {
        Self {
            error: problem.error,
            reason: problem.reason,
            origin: progress.origin(),
            admitted: progress.has_admission(),
            network_bytes: progress.network_bytes(),
            cancelled: false,
            superseded: false,
        }
    }

    pub(in crate::segmented) fn preflight(error: anyhow::Error, reason: ErrorReason) -> Self {
        Self {
            error,
            reason,
            origin: None,
            admitted: false,
            network_bytes: 0,
            cancelled: false,
            superseded: false,
        }
    }

    pub(in crate::segmented) fn admitted(
        error: anyhow::Error,
        reason: ErrorReason,
        origin: OriginTelemetry,
        network_bytes: u64,
    ) -> Self {
        Self {
            error,
            reason,
            origin: Some(origin),
            admitted: true,
            network_bytes,
            cancelled: false,
            superseded: false,
        }
    }

    pub(in crate::segmented) fn cancelled(
        origin: Option<OriginTelemetry>,
        network_bytes: u64,
    ) -> Self {
        Self {
            error: anyhow::anyhow!("HLS bootstrap cancelled"),
            reason: ErrorReason::Policy,
            admitted: origin.is_some(),
            origin,
            network_bytes,
            cancelled: true,
            superseded: false,
        }
    }

    pub(in crate::segmented) fn superseded(origin: OriginTelemetry, network_bytes: u64) -> Self {
        Self {
            error: anyhow::anyhow!("HLS bootstrap publication superseded"),
            reason: ErrorReason::Policy,
            origin: Some(origin),
            admitted: true,
            network_bytes,
            cancelled: false,
            superseded: true,
        }
    }

    pub(in crate::segmented) const fn origin(&self) -> Option<OriginTelemetry> {
        self.origin
    }

    pub(in crate::segmented) const fn reason(&self) -> ErrorReason {
        self.reason
    }

    pub(in crate::segmented) const fn network_bytes(&self) -> u64 {
        self.network_bytes
    }

    pub(in crate::segmented) const fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub(in crate::segmented) const fn is_superseded(&self) -> bool {
        self.superseded
    }

    pub(in crate::segmented) fn actual_resources(&self) -> Option<ResourceCost> {
        self.admitted
            .then(|| ResourceCost::new(self.network_bytes, 0, 0, 1))
    }
}

impl Display for FetchFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, formatter)
    }
}

pub(super) struct FetchProblem {
    error: anyhow::Error,
    reason: ErrorReason,
}

impl FetchProblem {
    pub(super) const fn new(error: anyhow::Error, reason: ErrorReason) -> Self {
        Self { error, reason }
    }

    pub(super) fn transport(error: anyhow::Error) -> Self {
        let reason = transport_reason(&error);
        Self::new(error, reason)
    }
}

impl Display for FetchProblem {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, formatter)
    }
}

fn transport_reason(error: &anyhow::Error) -> ErrorReason {
    for cause in error.chain() {
        if cause.is::<MediaRequestAdmissionTimeout>() {
            return ErrorReason::Timeout;
        }
        let Some(request) = cause.downcast_ref::<reqwest::Error>() else {
            continue;
        };
        if request.is_timeout() {
            return ErrorReason::Timeout;
        }
        if let Some(status) = request.status() {
            return http_reason(status);
        }
        return match request.is_connect() {
            true => ErrorReason::Connection,
            false => ErrorReason::InvalidResponse,
        };
    }
    ErrorReason::InvalidResponse
}

pub(super) fn http_reason(status: reqwest::StatusCode) -> ErrorReason {
    if status.is_server_error() {
        return ErrorReason::Http5xx;
    }
    if status.is_client_error() {
        return ErrorReason::Http4xx;
    }
    ErrorReason::InvalidResponse
}
