use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_index::{index_event, new_native_video_index};

fn event(keys: &Keys, caption: &str) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), caption)
        .tags([Tag::parse([
            "imeta".to_owned(),
            "url https://media.example/shared.mp4".to_owned(),
            format!("x {}", "a".repeat(64)),
            "m video/mp4".to_owned(),
        ])
        .expect("video tag")])
        .sign_with_keys(keys)
        .expect("signed event")
}

#[tokio::test]
async fn keeps_distinct_posts_that_share_the_same_media_blob() {
    let index = new_native_video_index();

    index_event(&event(&Keys::generate(), "first"), &index).await;
    index_event(&event(&Keys::generate(), "second"), &index).await;

    assert_eq!(index.ordered_ids().await.len(), 2);
}
