//! Typed session-reset failures retain stable caller-facing messages.

use rust_lib_ghostr::api::session_control::NostrSessionResetError;

#[test]
fn session_reset_errors_explain_the_failed_precondition() {
    assert_eq!(
        NostrSessionResetError::EngineNotStarted.to_string(),
        "the Nostr engine is not started"
    );
    assert_eq!(
        NostrSessionResetError::InvalidExpectedPublicKey.to_string(),
        "the expected Nostr public key is invalid"
    );
}
