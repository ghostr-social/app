use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

fn revision(keys: &Keys, created_at: u64, with_video: bool) -> nostr_sdk::Event {
    let mut tags = vec![Tag::parse(["d", "clip"]).expect("identifier")];
    if with_video {
        tags.push(
            Tag::parse([
                "imeta",
                "url https://media.example/video.mp4",
                "m video/mp4",
            ])
            .expect("video metadata"),
        );
    }
    EventBuilder::new(Kind::Custom(34236), format!("revision {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .tags(tags)
        .sign_with_keys(keys)
        .expect("signed revision")
}

#[tokio::test]
async fn empty_revision_removes_media_and_suppresses_older_replays() {
    let index = NativeVideoIndex::new(1);
    let keys = Keys::generate();
    let old = revision(&keys, 10, true);
    index.record(&old).await;

    index.record(&revision(&keys, 20, false)).await;
    index.record(&old).await;

    assert!(index.ordered_videos().await.is_empty());
    index.record(&revision(&keys, 30, true)).await;
    let videos = index.ordered_videos().await;
    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.created_at, 30);
}
