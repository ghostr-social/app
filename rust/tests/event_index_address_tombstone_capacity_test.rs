use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

fn revision(keys: &Keys, identifier: &str) -> Event {
    EventBuilder::new(Kind::Custom(34236), identifier)
        .custom_created_at(Timestamp::from(10))
        .tags([
            Tag::parse(["d", identifier]).expect("identifier"),
            Tag::parse([
                "imeta",
                "url https://media.example/video.mp4",
                "m video/mp4",
            ])
            .expect("video metadata"),
        ])
        .sign_with_keys(keys)
        .expect("signed revision")
}

#[tokio::test]
async fn bounds_address_deletion_tombstones_by_inventory_capacity() {
    let author = Keys::generate();
    let revisions = ["a", "b", "c"].map(|identifier| revision(&author, identifier));
    let public_key = author.public_key().to_hex();
    let tags = ["a", "b", "c"].map(|identifier| {
        let coordinate = format!("34236:{public_key}:{identifier}");
        Tag::parse(["a".to_owned(), coordinate]).expect("address reference")
    });
    let deletion = EventBuilder::new(Kind::EventDeletion, "removed")
        .custom_created_at(Timestamp::from(20))
        .tags(tags)
        .sign_with_keys(&author)
        .expect("signed deletion");
    let index = NativeVideoIndex::new(1);

    index.record(&deletion).await;
    for revision in &revisions {
        index.record(revision).await;
    }

    let videos = index.ordered_videos().await;
    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.identifier.as_deref(), Some("c"));
}
