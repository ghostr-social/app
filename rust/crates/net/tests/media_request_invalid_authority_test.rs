mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use request_gate_fixture::LocalMediaClient;

#[test]
fn invalid_or_credentialed_authority_fails_before_admission() {
    let requests = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );

    assert!(requests
        .get("ftp://media.example/video", PreemptionAuthority::Transition)
        .is_err());
    assert!(requests
        .get(
            "https://user:secret@media.example/video",
            PreemptionAuthority::Transition,
        )
        .is_err());
}
