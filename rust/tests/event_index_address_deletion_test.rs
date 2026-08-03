use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

fn revision(keys: &Keys, created_at: u64) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(34236), format!("revision {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .tags([
            Tag::parse(["d", "clip"]).expect("identifier"),
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

fn deletion(keys: &Keys, coordinate: String) -> nostr_sdk::Event {
    EventBuilder::new(Kind::EventDeletion, "removed")
        .custom_created_at(Timestamp::from(20))
        .tag(Tag::parse(["a".to_owned(), coordinate]).expect("address reference"))
        .sign_with_keys(keys)
        .expect("signed deletion")
}

#[tokio::test]
async fn address_deletion_suppresses_only_revisions_at_or_before_its_timestamp() {
    let author = Keys::generate();
    let old = revision(&author, 10);
    let coordinate = format!("34236:{}:clip", author.public_key().to_hex());
    let request = deletion(&author, coordinate);
    let index = NativeVideoIndex::new(8);

    index.record(&request).await;
    index.record(&old).await;
    assert!(index.ordered_videos().await.is_empty());

    index.record(&revision(&author, 21)).await;
    let videos = index.ordered_videos().await;
    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.created_at, 21);
}
