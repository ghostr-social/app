use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::NativeVideoIndex;

#[tokio::test]
async fn ignores_malformed_and_foreign_deletion_references() {
    let author = Keys::generate();
    let media = Tag::parse([
        "imeta",
        "url https://media.example/video.mp4",
        "m video/mp4",
    ])
    .expect("video metadata");
    let target = EventBuilder::new(Kind::Custom(22), "clip")
        .custom_created_at(Timestamp::from(10))
        .tag(media)
        .sign_with_keys(&author)
        .expect("signed video");
    let public_key = author.public_key().to_hex();
    let foreign_key = Keys::generate().public_key().to_hex();
    let deletion = EventBuilder::new(Kind::EventDeletion, "invalid references")
        .custom_created_at(Timestamp::from(20))
        .tags([
            Tag::parse(["e", "not-an-event-id"]).expect("malformed event reference"),
            Tag::parse(["a", &format!("34236:{foreign_key}:clip")]).expect("foreign address"),
            Tag::parse(["a", &format!("1:{public_key}:clip")]).expect("wrong kind"),
            Tag::parse(["a", &format!("34236:{public_key}:")]).expect("empty identifier"),
            Tag::parse(["a".to_owned(), "x".repeat(1_025)]).expect("oversized address"),
            Tag::parse(["p".to_owned(), public_key]).expect("unrelated tag"),
        ])
        .sign_with_keys(&author)
        .expect("signed deletion");
    let index = NativeVideoIndex::new(8);

    index.record(&target).await;
    index.record(&deletion).await;

    assert_eq!(index.ordered_videos().await.len(), 1);
}
