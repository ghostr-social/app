use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::{index_event, new_native_video_index};

fn event(keys: &Keys, created_at: u64, digest: char) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(34236), format!("revision {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .tags([
            Tag::parse(["d", "daily-clip"]).expect("identifier"),
            Tag::parse([
                "imeta".to_owned(),
                format!("url https://media.example/{digest}.mp4"),
                format!("x {}", digest.to_string().repeat(64)),
                "m video/mp4".to_owned(),
            ])
            .expect("video tag"),
        ])
        .sign_with_keys(keys)
        .expect("signed event")
}

#[tokio::test]
async fn replaces_an_addressable_post_with_its_newest_revision() {
    let index = new_native_video_index();
    let keys = Keys::generate();

    index_event(&event(&keys, 10, 'a'), &index).await;
    index_event(&event(&keys, 20, 'b'), &index).await;

    let videos = index.ordered_videos().await;
    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].identity.created_at, 20);
    assert_eq!(index.ordered_ids().await.len(), 1);
}
