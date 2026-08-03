use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

fn video(keys: &Keys, name: &str) -> Event {
    let media = Tag::parse([
        "imeta".to_owned(),
        format!("url https://media.example/{name}.mp4"),
        "m video/mp4".to_owned(),
    ])
    .expect("video metadata");
    EventBuilder::new(Kind::Custom(22), name)
        .custom_created_at(Timestamp::from(10))
        .tag(media)
        .sign_with_keys(keys)
        .expect("signed video")
}

#[tokio::test]
async fn bounds_exact_deletion_tombstones_by_inventory_capacity() {
    let author = Keys::generate();
    let targets = ["one", "two", "three"].map(|name| video(&author, name));
    let tags = targets
        .iter()
        .map(|event| Tag::parse(["e".to_owned(), event.id.to_hex()]).expect("event reference"));
    let deletion = EventBuilder::new(Kind::EventDeletion, "removed")
        .custom_created_at(Timestamp::from(20))
        .tags(tags)
        .sign_with_keys(&author)
        .expect("signed deletion");
    let mut ids = targets
        .iter()
        .map(|event| event.id.to_hex())
        .collect::<Vec<_>>();
    ids.sort();
    let index = NativeVideoIndex::new(1);

    index.record(&deletion).await;
    for target in &targets {
        index.record(target).await;
    }

    let videos = index.ordered_videos().await;
    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.event_id, ids[2]);
}
