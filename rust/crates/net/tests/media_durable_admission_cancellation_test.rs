mod durable_admission_fixture;
mod request_gate_fixture;
#[path = "durable_admission_fixture/worker.rs"]
mod worker;

use core::time::Duration;
use durable_admission_fixture::{executor, runtime, LedgerDirectory};
use ghostr_engine::{adaptive::PreemptionAuthority, RequestAuthority};
use ghostr_net::internet_allowance::InternetAllowance;
use ghostr_net::media_request_executor::{MediaRequestAdmissionTimeout, MediaRequestExecutor};
use worker::hold_blocking_worker;

const SOURCE: &str = "http://127.0.0.1:9/media";

#[test]
fn cancelled_accounting_keeps_its_slot_until_settlement_and_refunds_unsent_bytes() {
    let (_directory, ledger) = LedgerDirectory::open();
    runtime().block_on(check_cancellation(executor(ledger.clone()), ledger));
}

async fn check_cancellation(requests: MediaRequestExecutor, ledger: InternetAllowance) {
    let held = hold_blocking_worker();
    let authority = RequestAuthority::from_url(SOURCE).expect("fixture authority");
    for _ in 0..2 {
        let result = admission(&requests, Duration::from_millis(20)).await;
        assert!(
            result
                .err()
                .is_some_and(|error| error.is::<MediaRequestAdmissionTimeout>()),
            "a cancelled durable reservation must remain bounded by its gate slot"
        );
        assert_eq!(
            requests.active_for(&authority),
            1,
            "cleanup owns the sole slot"
        );
    }
    drop(held);
    let next = admission(&requests, Duration::from_secs(180))
        .await
        .expect("capacity reclaimed");
    assert_eq!(
        ledger.usage(),
        (0, 1),
        "unsent cancellation refunds exactly its reservation"
    );
    drop(next);
}

async fn admission(
    requests: &MediaRequestExecutor,
    wait: Duration,
) -> anyhow::Result<ghostr_net::media_request_executor::AdmittedMediaRequest> {
    requests
        .get(SOURCE, PreemptionAuthority::PlaybackCritical)
        .expect("fixture request")
        .body_limit(1)
        .admit_for(wait)
        .await
}
