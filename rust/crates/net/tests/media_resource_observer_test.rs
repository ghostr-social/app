mod request_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    MediaRequestExecutor, MediaRequestLimits, MediaResourceObserver,
};
use request_gate_fixture::{HeldOrigin, LocalMediaClient};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Default)]
struct Counts {
    requests: AtomicU64,
    bytes: AtomicU64,
}

impl MediaResourceObserver for Counts {
    fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    fn record_response_bytes(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn observer_counts_the_attempt_and_each_received_body_chunk() {
    let mut origin = HeldOrigin::serve().await;
    let (requests, counts) = executor();
    let sending = tokio::spawn(open(requests, origin.url.clone()));
    origin.expect_hit().await;
    assert_eq!(counts.requests.load(Ordering::Relaxed), 1);
    assert_eq!(counts.bytes.load(Ordering::Relaxed), 0);

    origin.release_one();
    let mut response = sending.await.unwrap().unwrap();
    assert_eq!(response.chunk().await.unwrap().unwrap().len(), 1);
    assert_eq!(counts.bytes.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn a_failed_send_still_counts_as_a_request_attempt() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/media", listener.local_addr().unwrap());
    drop(listener);
    let (requests, counts) = executor();

    assert!(open(requests, url).await.is_err());
    assert_eq!(counts.requests.load(Ordering::Relaxed), 1);
    assert_eq!(counts.bytes.load(Ordering::Relaxed), 0);
}

fn executor() -> (MediaRequestExecutor, Arc<Counts>) {
    let executor = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let counts = Arc::new(Counts::default());
    assert!(executor.install_resource_observer(counts.clone()));
    (executor, counts)
}

#[test]
fn observer_install_has_one_owner_across_executor_clones() {
    let executor = MediaRequestExecutor::new(
        LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    assert!(executor.install_resource_observer(Arc::new(Counts::default())));
    assert!(!executor
        .clone()
        .install_resource_observer(Arc::new(Counts::default())));
}

async fn open(
    requests: MediaRequestExecutor,
    url: String,
) -> anyhow::Result<ghostr_net::media_request_executor::MediaResponse> {
    requests
        .get(&url, PreemptionAuthority::Transition)?
        .admit()
        .await?
        .send()
        .await
}
