use nostr_sdk::{EventBuilder, Keys, Kind, RelayPoolNotification, RelayUrl, SubscriptionId, Tag};
use rust_lib_ghostr::video::event_index::{run_event_identity_indexer, NativeVideoIndex};
use tokio::sync::broadcast;

fn video_event() -> nostr_sdk::Event {
    let media = Tag::parse([
        "imeta",
        "url https://media.example/video.mp4",
        "m video/mp4",
    ])
    .expect("video tag");
    EventBuilder::new(Kind::Custom(22), "clip")
        .tag(media)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

#[tokio::test]
async fn continues_indexing_after_the_notification_receiver_lags() {
    let (sender, receiver) = broadcast::channel(1);
    sender.send(RelayPoolNotification::Shutdown).expect("first");
    sender
        .send(RelayPoolNotification::Shutdown)
        .expect("second");
    sender
        .send(RelayPoolNotification::Event {
            relay_url: RelayUrl::parse("wss://relay.example").expect("relay URL"),
            subscription_id: SubscriptionId::new("videos"),
            event: Box::new(video_event()),
        })
        .expect("video");
    drop(sender);
    let index = NativeVideoIndex::new(8);

    run_event_identity_indexer(receiver, index.clone()).await;

    assert_eq!(index.ordered_videos().await.len(), 1);
}
