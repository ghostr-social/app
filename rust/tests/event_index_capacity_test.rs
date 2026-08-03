use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::{index_event, NativeVideoIndex};

fn event(created_at: u64, media: &[&str]) -> nostr_sdk::Event {
    let tags = media.iter().enumerate().map(|(index, name)| {
        Tag::parse([
            "imeta".to_owned(),
            format!("url https://media.example/{name}.mp4"),
            format!("x {created_at:032x}{index:032x}"),
            "m video/mp4".to_owned(),
        ])
        .expect("video tag")
    });
    EventBuilder::new(Kind::Custom(22), format!("clip {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

#[tokio::test]
async fn keeps_newest_media_within_the_hard_inventory_capacity() {
    let videos = NativeVideoIndex::new(2);

    for item in [
        event(30, &["newest-primary", "newest-alternate"]),
        event(20, &["middle"]),
        event(10, &["oldest"]),
    ] {
        index_event(&item, &videos).await;
    }

    let values = videos.ordered_videos().await;
    assert_eq!(values.len(), 2);
    assert_eq!(
        values
            .iter()
            .map(|value| value.identity.created_at)
            .collect::<Vec<_>>(),
        [30, 30]
    );
    assert_eq!(
        values
            .iter()
            .map(|value| value.video.url.as_str())
            .collect::<Vec<_>>(),
        [
            "https://media.example/newest-primary.mp4",
            "https://media.example/newest-alternate.mp4",
        ]
    );
}

#[tokio::test]
async fn accepts_at_most_five_valid_media_rows_from_one_event() {
    let videos = NativeVideoIndex::new(128);

    index_event(
        &event(30, &["one", "two", "three", "four", "five", "six", "seven"]),
        &videos,
    )
    .await;

    let values = videos.ordered_videos().await;
    assert_eq!(values.len(), 5);
    assert_eq!(
        values
            .iter()
            .map(|value| value.video.url.as_str())
            .collect::<Vec<_>>(),
        [
            "https://media.example/one.mp4",
            "https://media.example/two.mp4",
            "https://media.example/three.mp4",
            "https://media.example/four.mp4",
            "https://media.example/five.mp4",
        ]
    );
}
