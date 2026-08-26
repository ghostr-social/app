mod redirect_gate_fixture;

use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    MediaRequestExecutor, MediaRequestLimits, MediaResourceObserver,
};
use redirect_gate_fixture::chain::redirect_chain;
use redirect_gate_fixture::OneHopClient;
use std::sync::Arc;

#[derive(Default)]
struct Counts {
    attempts: AtomicU64,
    bytes: AtomicU64,
}

impl MediaResourceObserver for Counts {
    fn record_request(&self) {
        self.attempts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_response_bytes(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn two_redirects_and_the_final_response_count_three_attempts() {
    let url = redirect_chain(2).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let counts = Arc::new(Counts::default());
    assert!(executor.install_resource_observer(std::sync::Arc::<Counts>::clone(&counts)));

    let mut response = executor
        .get(&url, PreemptionAuthority::Transition)
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

    assert_eq!(counts.attempts.load(Ordering::Relaxed), 3);
    assert_eq!(counts.bytes.load(Ordering::Relaxed), 1);
}
