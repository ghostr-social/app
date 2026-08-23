//! Failure triage for media transfers. Some failures are about the
//! moment — a timeout, a 5xx, a reset connection — and deserve another
//! attempt shortly. Others are about the source itself: a host that
//! does not resolve, a URL that is gone, a certificate that cannot be
//! trusted, a destination the SSRF guard refuses. The retry policy
//! spends far fewer attempts on the second kind, because the engine
//! exists to spend the radio carefully (plan §3).

use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;
use ghostr_net::native_cache_failure::PermanentCacheFailure;
use ghostr_net::origin_content_type::UnsupportedOriginMediaType;
use reqwest::StatusCode;

#[cfg(test)]
#[path = "failure/origin_failure_test.rs"]
mod origin_failure_test;
#[cfg(test)]
#[path = "failure/whole_body_limit_test.rs"]
mod whole_body_limit_test;

/// How hopeless one failed attempt against a source looks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureClass {
    /// The source itself looks wrong: retrying it soon cannot help.
    Permanent,
    /// The moment looks wrong, not the source.
    Transient,
}

/// Text that appears in some link of the error chain when name
/// resolution or TLS trust failed. Both say "this source", not "this
/// moment", so they are triaged as permanent-ish.
const HOPELESS: &[&str] = &[
    "dns error",
    "failed to lookup address",
    "name or service not known",
    "nodename nor servname",
    "no such host",
    "temporary failure in name resolution",
    "invalid peer certificate",
    "certificate verify failed",
    "certificate has expired",
    "tls handshake",
];

/// Triages one failed chunk transfer or probe.
pub fn classify(error: &anyhow::Error) -> FailureClass {
    if error.downcast_ref::<PermanentCacheFailure>().is_some() {
        return FailureClass::Permanent;
    }
    if error.downcast_ref::<UnsupportedOriginMediaType>().is_some() {
        return FailureClass::Permanent;
    }
    if let Some(status) = rejected_status(error) {
        return of_status(status);
    }
    match error.chain().any(|cause| is_hopeless(&cause.to_string())) {
        true => FailureClass::Permanent,
        false => FailureClass::Transient,
    }
}

/// Returns remote failure evidence only after a request reached an origin.
pub(crate) fn origin_failure_class(error: &anyhow::Error) -> Option<FailureClass> {
    (!error.is::<MediaRequestAdmissionTimeout>()
        && !crate::chunk::sink::is_local_store_failure(error)
        && !crate::chunk::whole_body_policy::is(error))
    .then(|| classify(error))
}

/// The HTTP status a rejected request carried, if it got that far.
fn rejected_status(error: &anyhow::Error) -> Option<StatusCode> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .and_then(reqwest::Error::status)
}

/// A 4xx says the URL is wrong; `408` and `429` are the two that
/// explicitly ask for another attempt later. 5xx is the server's bad
/// moment, not a bad source.
fn of_status(status: StatusCode) -> FailureClass {
    let retryable =
        status == StatusCode::REQUEST_TIMEOUT || status == StatusCode::TOO_MANY_REQUESTS;
    match status.is_client_error() && !retryable {
        true => FailureClass::Permanent,
        false => FailureClass::Transient,
    }
}

fn is_hopeless(message: &str) -> bool {
    let lowered = message.to_ascii_lowercase();
    HOPELESS.iter().any(|marker| lowered.contains(marker))
}
