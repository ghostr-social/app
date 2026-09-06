#![cfg(unix)]

mod durable_admission_fixture;
#[path = "durable_admission_fixture/read_only.rs"]
mod read_only;
mod request_gate_fixture;

use durable_admission_fixture::{executor, runtime, LedgerDirectory};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::internet_allowance::InternetAdmissionDenied;
use ghostr_net::media_request_executor::MediaRequestExecutor;

#[test]
fn zero_body_admission_needs_no_ledger_write_but_a_failed_payload_write_closes_admission() {
    let (directory, ledger) = LedgerDirectory::open();
    let _read_only = directory.read_only();
    runtime().block_on(check_head_then_payload(executor(ledger)));
}

async fn check_head_then_payload(requests: MediaRequestExecutor) {
    let head = request(&requests)
        .head()
        .admit()
        .await
        .expect("HEAD changes no body accounting");
    drop(head);
    let failed = request(&requests).body_limit(1).admit().await;
    assert!(
        failed
            .err()
            .is_some_and(|error| error.is::<InternetAdmissionDenied>()),
        "payload cannot bypass failed durable accounting"
    );
    let after = request(&requests).head().admit().await;
    assert!(
        after
            .err()
            .is_some_and(|error| error.is::<InternetAdmissionDenied>()),
        "failed accounting cannot be reopened by a zero-byte request"
    );
}

fn request(requests: &MediaRequestExecutor) -> ghostr_net::media_request_executor::MediaRequest {
    requests
        .get(
            "http://127.0.0.1:9/media",
            PreemptionAuthority::PlaybackCritical,
        )
        .expect("fixture request")
}
