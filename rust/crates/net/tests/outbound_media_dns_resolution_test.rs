use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use ghostr_net::outbound_media_client::MediaHttpClient;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct RecordingResolver {
    address: SocketAddr,
    calls: Arc<AtomicUsize>,
}

impl Resolve for RecordingResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let addresses: Addrs = Box::new(vec![self.address].into_iter());
        Box::pin(async move { Ok(addresses) })
    }
}

#[tokio::test]
async fn allows_a_hostname_only_after_its_resolution_is_public() {
    let calls = Arc::new(AtomicUsize::new(0));
    let resolver = RecordingResolver {
        address: "1.1.1.1:9".parse().expect("public address"),
        calls: calls.clone(),
    };
    let client = MediaHttpClient::with_resolver(Arc::new(resolver)).expect("media client");
    let request = client.get("http://media.test/video.mp4").expect("request");

    let result = tokio::time::timeout(Duration::from_millis(100), request.send()).await;

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    if let Ok(Err(error)) = result {
        assert!(!has_message(&error, "media host has no public address"));
    }
}

#[tokio::test]
async fn system_resolution_rejects_localhost_before_connecting() {
    let client = MediaHttpClient::public().expect("media client");
    let request = client.get("http://localhost:9/video.mp4").expect("request");

    let result = tokio::time::timeout(Duration::from_secs(1), request.send())
        .await
        .expect("localhost resolution timed out");

    assert!(result.is_err());
}

fn has_message(mut error: &(dyn Error + 'static), expected: &str) -> bool {
    loop {
        if error.to_string().contains(expected) {
            return true;
        }
        let Some(source) = error.source() else {
            return false;
        };
        error = source;
    }
}
