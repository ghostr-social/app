use super::super::telemetry::{FetchProblem, FetchProgress};
use super::super::FetchFailure;
use ghostr_net::internet_allowance::InternetAdmissionDenied;

#[test]
fn allowance_denial_during_redirect_is_retry_and_origin_neutral() {
    let problem = FetchProblem::transport(anyhow::Error::new(InternetAdmissionDenied));
    let failure = FetchFailure::new(problem, &FetchProgress::default());

    assert!(failure.retry_class().is_none());
    assert!(!failure.records_origin_evidence());
}
