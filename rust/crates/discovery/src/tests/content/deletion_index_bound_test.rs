use crate::content::deletions::{deletion_claims, DeletionIndex};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

#[test]
fn deletion_index_compacts_repeats_and_bounds_unique_targets() {
    let keys = Keys::generate();
    let mut index = DeletionIndex::with_retention(3);

    index.ingest(deletion_claims(&[deletion(&keys, "same", 10)]));
    index.ingest(deletion_claims(&[deletion(&keys, "same", 20)]));
    assert_eq!(index.retained_claims(), 1);

    for (target, created_at) in [("one", 30), ("two", 40), ("three", 50)] {
        index.ingest(deletion_claims(&[deletion(&keys, target, created_at)]));
    }

    assert_eq!(index.retained_claims(), 3);
}

fn deletion(keys: &Keys, target: &str, created_at: u64) -> Event {
    let tag = Tag::parse(vec!["e".to_owned(), target.to_owned()]).expect("tag");
    EventBuilder::new(Kind::EventDeletion, "delete")
        .tags([tag])
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(keys)
        .expect("deletion")
}
