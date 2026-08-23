use super::error_reason;
use crate::chunk::downloader::ResponseFailure;
use anyhow::Context;
use ghostr_engine::origin_model::ErrorReason;

#[test]
fn typed_response_failures_do_not_depend_on_error_wording() {
    let invalid = Err::<(), _>(anyhow::anyhow!("opaque"))
        .context(ResponseFailure::InvalidResponse)
        .unwrap_err();
    let range = Err::<(), _>(anyhow::anyhow!("opaque"))
        .context(ResponseFailure::RangeNoncompliant)
        .unwrap_err();

    assert_eq!(error_reason(&invalid), ErrorReason::InvalidResponse);
    assert_eq!(error_reason(&range), ErrorReason::RangeNoncompliant);
}
