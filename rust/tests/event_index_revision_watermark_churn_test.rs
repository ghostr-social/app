use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

fn revision(keys: &Keys, identifier: &str, created_at: u64, media: bool) -> nostr_sdk::Event {
    let mut tags = vec![Tag::parse(["d", identifier]).expect("identifier")];
    if media {
        tags.push(
            Tag::parse([
                "imeta",
                "url https://media.example/video.mp4",
                "m video/mp4",
            ])
            .expect("video metadata"),
        );
    }
    EventBuilder::new(Kind::Custom(34236), "revision")
        .custom_created_at(Timestamp::from(created_at))
        .tags(tags)
        .sign_with_keys(keys)
        .expect("signed revision")
}

#[tokio::test]
async fn active_media_still_blocks_old_replays_after_watermark_churn() {
    let index = NativeVideoIndex::new(1);
    let keys = Keys::generate();
    index.record(&revision(&keys, "active", 100, true)).await;
    index.record(&revision(&keys, "empty", 200, false)).await;

    index.record(&revision(&keys, "active", 50, true)).await;

    let videos = index.ordered_videos().await;
    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.created_at, 100);
}
