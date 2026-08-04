use nostr_sdk::{EventBuilder, Keys, Kind, RelayPoolNotification, RelayUrl, SubscriptionId, Tag};
use rust_lib_ghostr::video::event_index::{new_native_video_index, run_event_identity_indexer};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

/// Regression pin for plan §4 step 10: the viewer-blind freelancer is
/// retired, so indexing a discovered video must never contact its host.
/// Downloads are granted only by the focus-driven delivery manager.
#[tokio::test(start_paused = true)]
async fn indexing_alone_schedules_no_downloads() {
    let (url, connections) = spawn_counting_media_server().await;
    let (sender, receiver) = broadcast::channel(4);
    sender
        .send(RelayPoolNotification::Event {
            relay_url: RelayUrl::parse("wss://relay.example").expect("relay URL"),
            subscription_id: SubscriptionId::new("videos"),
            event: Box::new(video_event(&url)),
        })
        .expect("deliver video event");
    drop(sender);
    let index = new_native_video_index();

    run_event_identity_indexer(receiver, index.clone()).await;
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert_eq!(index.ordered_videos().await.len(), 1, "video must be indexed");
    assert_eq!(
        connections.load(Ordering::SeqCst),
        0,
        "indexing alone must not open any connection to the media host"
    );
}

async fn spawn_counting_media_server() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let connections = Arc::new(AtomicUsize::new(0));
    let counter = connections.clone();
    tokio::spawn(async move {
        while listener.accept().await.is_ok() {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    });
    (format!("http://{address}/video.mp4"), connections)
}

fn video_event(url: &str) -> nostr_sdk::Event {
    let media = Tag::parse([
        "imeta".to_owned(),
        format!("url {url}"),
        "m video/mp4".to_owned(),
    ])
    .expect("video tag");
    EventBuilder::new(Kind::Custom(22), "clip")
        .tag(media)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}
