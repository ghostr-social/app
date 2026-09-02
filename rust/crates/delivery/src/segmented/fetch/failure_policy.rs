use crate::manager::failure::FailureClass;
use ghostr_engine::origin_model::ErrorReason;
use reqwest::StatusCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FailureDisposition {
    Retry(FailureClass),
    Requeue,
    RestartObject,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureScope {
    Local,
    Origin,
}

#[derive(Clone, Copy)]
pub(super) struct FailurePolicy {
    pub(super) disposition: FailureDisposition,
    scope: FailureScope,
}

impl FailurePolicy {
    pub(super) const fn neutral() -> Self {
        Self {
            disposition: FailureDisposition::Requeue,
            scope: FailureScope::Local,
        }
    }

    pub(super) const fn terminal() -> Self {
        Self {
            disposition: FailureDisposition::Terminal,
            scope: FailureScope::Local,
        }
    }

    pub(super) const fn restart_object() -> Self {
        Self {
            disposition: FailureDisposition::RestartObject,
            scope: FailureScope::Local,
        }
    }

    pub(super) const fn for_reason(reason: ErrorReason) -> Self {
        match reason {
            ErrorReason::Timeout | ErrorReason::Http5xx | ErrorReason::Connection => {
                Self::retry(FailureClass::Transient)
            }
            ErrorReason::Dns
            | ErrorReason::Tls
            | ErrorReason::Http4xx
            | ErrorReason::RangeNoncompliant
            | ErrorReason::InvalidResponse => Self::retry(FailureClass::Permanent),
            // A media-policy verdict is deterministic and local: retrying it
            // only delays the terminal `failed` evidence the feed rescues on.
            ErrorReason::Policy => Self::terminal(),
            ErrorReason::Unknown => Self::retry(FailureClass::Transient),
        }
    }

    pub(super) fn for_status(status: StatusCode) -> Self {
        let retryable = status.as_u16() == 408 || status.as_u16() == 429;
        if status.is_server_error() || retryable {
            return Self::retry(FailureClass::Transient);
        }
        Self::retry(FailureClass::Permanent)
    }

    const fn retry(class: FailureClass) -> Self {
        Self {
            disposition: FailureDisposition::Retry(class),
            scope: FailureScope::Origin,
        }
    }

    pub(super) const fn records_origin_evidence(self) -> bool {
        matches!(self.scope, FailureScope::Origin)
    }
}
