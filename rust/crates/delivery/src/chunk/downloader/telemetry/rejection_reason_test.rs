use super::error_reason;
use crate::chunk::downloader::ResponseFailure;
use anyhow::Context as _;
use ghostr_engine::origin_model::ErrorReason;

#[test]
fn typed_response_failures_do_not_depend_on_error_wording() {
    let invalid = Err::<(), _>(anyhow::anyhow!("opaque"))
        .context(ResponseFailure::InvalidResponse)
        .expect_err("scenario must fail");
    let range = Err::<(), _>(anyhow::anyhow!("opaque"))
        .context(ResponseFailure::RangeNoncompliant)
        .expect_err("scenario must fail");

    assert_eq!(error_reason(&invalid), ErrorReason::InvalidResponse);
    assert_eq!(error_reason(&range), ErrorReason::RangeNoncompliant);
}
