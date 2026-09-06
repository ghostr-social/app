mod durable_admission_fixture;
mod request_gate_fixture;
#[path = "durable_admission_fixture/worker.rs"]
mod worker;

use core::time::Duration;
use durable_admission_fixture::{executor, runtime, LedgerDirectory};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestAdmissionTimeout, MediaRequestExecutor};
use worker::hold_blocking_worker;

#[test]
fn durable_accounting_wait_honors_the_public_admission_deadline() {
    let (_directory, ledger) = LedgerDirectory::open();
    runtime().block_on(assert_admission_deadline(executor(ledger)));
}

async fn assert_admission_deadline(requests: MediaRequestExecutor) {
    let _release_worker = hold_blocking_worker();
    let result = requests
        .get(
            "http://127.0.0.1:9/media",
            PreemptionAuthority::PlaybackCritical,
        )
        .expect("fixture request")
        .body_limit(1)
        .admit_for(Duration::from_millis(20))
        .await;

    assert!(
        result
            .err()
            .is_some_and(|error| error.is::<MediaRequestAdmissionTimeout>()),
        "accounting work must not block the async admission deadline"
    );
}
