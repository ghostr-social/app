use nostr_sdk::{EventBuilder, Keys, Kind};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

#[tokio::test]
async fn ignores_events_outside_the_native_video_kinds() {
    let event = EventBuilder::new(Kind::TextNote, "not a video")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let index = NativeVideoIndex::new(8);

    index.record(&event).await;

    assert!(index.ordered_videos().await.is_empty());
}
