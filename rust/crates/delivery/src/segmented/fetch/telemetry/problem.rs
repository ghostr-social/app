use crate::segmented::fetch::failure_policy::FailurePolicy;
use core::fmt::{Display, Formatter};
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;
use reqwest::StatusCode;

pub(in crate::segmented::fetch) struct FetchProblem {
    pub(super) error: anyhow::Error,
    pub(super) reason: ErrorReason,
    pub(super) status: Option<StatusCode>,
    pub(super) policy: FailurePolicy,
}

impl FetchProblem {
    pub(in crate::segmented::fetch) const fn new(
        error: anyhow::Error,
        reason: ErrorReason,
    ) -> Self {
        Self::with_policy(error, reason, FailurePolicy::for_reason(reason))
    }

    pub(in crate::segmented::fetch) const fn neutral(
        error: anyhow::Error,
        reason: ErrorReason,
    ) -> Self {
        Self::with_policy(error, reason, FailurePolicy::neutral())
    }

    pub(in crate::segmented::fetch) const fn restart_object(
        error: anyhow::Error,
        reason: ErrorReason,
    ) -> Self {
        Self::with_policy(error, reason, FailurePolicy::restart_object())
    }

    pub(in crate::segmented::fetch) fn http(error: anyhow::Error, status: StatusCode) -> Self {
        Self {
            error,
            reason: http_reason(status),
            status: Some(status),
            policy: FailurePolicy::for_status(status),
        }
    }

    pub(in crate::segmented::fetch) fn transport(error: anyhow::Error) -> Self {
        if error.is::<ghostr_net::internet_allowance::InternetAdmissionDenied>() {
            return Self::neutral(error, ErrorReason::Policy);
        }
        if admission_timed_out(&error) {
            return Self::neutral(error, ErrorReason::Timeout);
        }
        if let Some(status) = transport_status(&error) {
            Self::http(error, status)
        } else {
            let reason = transport_reason(&error);
            Self::new(error, reason)
        }
    }

    const fn with_policy(error: anyhow::Error, reason: ErrorReason, policy: FailurePolicy) -> Self {
        Self {
            error,
            reason,
            status: None,
            policy,
        }
    }
}

impl Display for FetchProblem {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.error, formatter)
    }
}

fn admission_timed_out(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.is::<MediaRequestAdmissionTimeout>())
}

fn transport_reason(error: &anyhow::Error) -> ErrorReason {
    let text = format!("{error:#}").to_ascii_lowercase();
    if text.contains("dns") || text.contains("lookup address") {
        return ErrorReason::Dns;
    }
    if text.contains("certificate") || text.contains("tls") {
        return ErrorReason::Tls;
    }
    for cause in error.chain() {
        if let Some(request) = cause.downcast_ref::<reqwest::Error>() {
            return request_reason(request);
        }
    }
    ErrorReason::InvalidResponse
}

fn request_reason(request: &reqwest::Error) -> ErrorReason {
    if request.is_timeout() {
        return ErrorReason::Timeout;
    }
    if request.is_connect() {
        ErrorReason::Connection
    } else {
        ErrorReason::InvalidResponse
    }
}

fn transport_status(error: &anyhow::Error) -> Option<StatusCode> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .and_then(reqwest::Error::status)
}

fn http_reason(status: StatusCode) -> ErrorReason {
    if status.is_server_error() {
        return ErrorReason::Http5xx;
    }
    if status.is_client_error() {
        return ErrorReason::Http4xx;
    }
    ErrorReason::InvalidResponse
}
