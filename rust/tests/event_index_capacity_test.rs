use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_index::{index_event, NativeVideoIndex};

fn event(index: usize) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), format!("clip {index}"))
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url https://media.example/{index}.mp4"),
            format!("x {index:064x}"),
            "m video/mp4".to_owned(),
        ])
        .expect("video tag")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

#[tokio::test]
async fn evicts_the_oldest_post_at_the_native_inventory_capacity() {
    let videos = NativeVideoIndex::new(2);

    for index in 1..=3 {
        index_event(&event(index), &videos).await;
    }

    let values = videos.ordered_videos().await;
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].video.url, "https://media.example/2.mp4");
    assert_eq!(values[1].video.url, "https://media.example/3.mp4");
}
