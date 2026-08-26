mod redirect_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use redirect_gate_fixture::chain::redirect_chain;
use redirect_gate_fixture::OneHopClient;

fn executor() -> MediaRequestExecutor {
    MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    )
}

#[tokio::test]
async fn exactly_ten_redirects_succeed_and_the_eleventh_is_rejected() {
    let allowed = redirect_chain(10).await;
    let mut response = executor()
        .get(&allowed, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture");
    assert_eq!(
        response
            .chunk()
            .await
            .expect("valid test fixture")
            .expect("valid test fixture")
            .as_ref(),
        b"x"
    );

    let rejected = redirect_chain(11).await;
    let result = executor()
        .get(&rejected, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await;
    let Err(error) = result else {
        panic!("eleven redirects were accepted");
    };
    assert!(format!("{error:#}").contains("media redirect limit exceeded"));
}
