use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

fn video(keys: &Keys, created_at: u64) -> nostr_sdk::Event {
    let media = Tag::parse([
        "imeta",
        "url https://media.example/video.mp4",
        "m video/mp4",
    ])
    .expect("video metadata");
    EventBuilder::new(Kind::Custom(22), "clip")
        .custom_created_at(Timestamp::from(created_at))
        .tag(media)
        .sign_with_keys(keys)
        .expect("signed video")
}

fn deletion(keys: &Keys, target: &nostr_sdk::Event) -> nostr_sdk::Event {
    EventBuilder::new(Kind::EventDeletion, "removed")
        .custom_created_at(Timestamp::from(20))
        .tag(Tag::parse(["e".to_owned(), target.id.to_hex()]).expect("event reference"))
        .sign_with_keys(keys)
        .expect("signed deletion")
}

#[tokio::test]
async fn author_valid_exact_deletions_remove_and_suppress_video_events() {
    let author = Keys::generate();
    let target = video(&author, 10);
    let request = deletion(&author, &target);
    let existing = NativeVideoIndex::new(8);
    existing.record(&target).await;
    existing.record(&request).await;
    assert!(existing.ordered_videos().await.is_empty());

    let replayed = NativeVideoIndex::new(8);
    replayed.record(&request).await;
    replayed.record(&target).await;
    assert!(replayed.ordered_videos().await.is_empty());

    let forged = NativeVideoIndex::new(8);
    forged.record(&target).await;
    forged.record(&deletion(&Keys::generate(), &target)).await;
    assert_eq!(forged.ordered_videos().await.len(), 1);
}
