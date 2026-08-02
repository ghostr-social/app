use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_index::{index_event, new_native_video_index};

#[tokio::test]
async fn indexes_each_video_digest_once_in_discovery_order() {
    let hash = "a".repeat(64);
    let event = EventBuilder::new(Kind::Custom(22), "clip")
        .tags([Tag::parse([
            "imeta".to_owned(),
            "url https://media.example/video.mp4".to_owned(),
            format!("x {hash}"),
            "m video/mp4".to_owned(),
        ])
        .expect("video tag")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let index = new_native_video_index();

    index_event(&event, &index).await;
    index_event(&event, &index).await;

    assert_eq!(index.ordered_ids().await.len(), 1);
    assert_eq!(index.ordered_videos().await[0].video.id, hash);
}
