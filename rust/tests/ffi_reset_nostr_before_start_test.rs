//! Session reset is an engine operation and has a typed pre-start failure.

use rust_lib_ghostr::api::session_control::{ffi_reset_nostr_session, NostrSessionResetError};

#[tokio::test]
async fn reset_before_engine_start_reports_engine_not_started() {
    let result = ffi_reset_nostr_session(None).await;

    assert_eq!(result, Err(NostrSessionResetError::EngineNotStarted));
}

#[tokio::test]
async fn invalid_expected_account_is_rejected_at_the_ffi_boundary() {
    let result = ffi_reset_nostr_session(Some("not-a-public-key".to_owned())).await;

    assert_eq!(
        result,
        Err(NostrSessionResetError::InvalidExpectedPublicKey)
    );
}
