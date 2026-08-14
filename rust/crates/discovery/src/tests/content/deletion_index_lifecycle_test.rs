use crate::content::deletions::{deletion_claims, DeletionIndex};
use crate::content::parsing::video_post_from_event;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

#[test]
fn newest_deletion_survives_anchor_release_and_restoration() {
    let author = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(34235), "https://cdn.example/v.mp4")
        .tags([tag(&["d", "clip"])])
        .custom_created_at(Timestamp::from(15))
        .sign_with_keys(&author)
        .expect("original");
    let post = video_post_from_event(&original).expect("video");
    let mut index = DeletionIndex::with_retention(2);
    index.ingest(deletion_claims(&[deletion(&author, &original, 20)]));
    index.ingest(deletion_claims(&[deletion(&author, &original, 10)]));
    index.reanchor(std::slice::from_ref(&post));
    index.ingest(deletion_claims(&[deletion(&author, &original, 10)]));

    index.reanchor(&[]);
    index.reanchor(std::slice::from_ref(&post));

    assert!(index.deletes_content(&post));
    assert_eq!(index.retained_claims(), 1);
}

fn deletion(author: &Keys, target: &Event, created_at: u64) -> Event {
    let coordinate = format!("34235:{}:clip", target.pubkey);
    EventBuilder::new(Kind::EventDeletion, "delete")
        .tags([tag(&["a", &coordinate])])
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(author)
        .expect("deletion")
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("tag")
}
