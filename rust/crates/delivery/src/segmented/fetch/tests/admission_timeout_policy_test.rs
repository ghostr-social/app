use super::super::telemetry::{FetchProblem, FetchProgress};
use super::super::FetchFailure;
use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;

#[test]
fn typed_redirect_admission_timeout_is_retry_and_origin_neutral() {
    let problem = FetchProblem::transport(anyhow::Error::new(MediaRequestAdmissionTimeout));
    let failure = FetchFailure::new(problem, &FetchProgress::default());

    assert!(failure.retry_class().is_none());
    assert!(!failure.records_origin_evidence());
}
