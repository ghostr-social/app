use crate::content::deletions::{deletion_claims, DeletionIndex};
use crate::content::parsing::video_post_from_event;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

#[test]
fn one_author_cannot_evict_another_authors_active_tombstone() {
    let alice = Keys::generate();
    let mallory = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&alice)
        .expect("original");
    let post = video_post_from_event(&original).expect("video");
    let mut index = DeletionIndex::with_retention(4);
    index.reanchor(std::slice::from_ref(&post));
    index.ingest(deletion_claims(&[deletion(
        &alice,
        &original.id.to_hex(),
        1,
    )]));

    for created_at in 2..=5 {
        index.ingest(deletion_claims(&[deletion(
            &mallory,
            &format!("target-{created_at}"),
            created_at,
        )]));
    }

    assert!(index.deletes_content(&post));
}

fn deletion(keys: &Keys, target: &str, created_at: u64) -> Event {
    EventBuilder::new(Kind::EventDeletion, "delete")
        .tags([Tag::parse(vec!["e".to_owned(), target.to_owned()]).expect("tag")])
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(keys)
        .expect("deletion")
}
