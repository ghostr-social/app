//! Pages for a closed or unknown feed ID never recreate that feed.

use crate::content::deletions::deletion_claims;
use crate::feed::store::{FeedId, FeedStore};
use crate::tests::feed_support::{empty_graph, parsed, video_note};
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[test]
fn unknown_feed_ignores_fresh_and_older_pages() {
    let graph = empty_graph();
    let unknown = FeedId(u64::MAX);
    let mut store = FeedStore::new();
    let keys = Keys::generate();
    let post = parsed(&video_note(&keys, "late", 1));

    store.ingest_first_page(unknown, Vec::new(), &graph);
    let appended = store.ingest_older_page(unknown, Vec::new(), &graph);
    assert!(!store.ingest_progress(unknown, post.clone(), &graph));
    assert!(!store.ingest_head_page(unknown, vec![post], &graph));
    assert!(!store.ingest_deletions(unknown, Vec::new(), &graph));
    assert!(!store.ingest_deletions(unknown, deletion_claims(&[deletion(&keys)]), &graph,));

    assert!(store.posts(unknown).is_empty());
    assert!(!appended);
}

fn deletion(keys: &Keys) -> nostr_sdk::Event {
    EventBuilder::new(Kind::EventDeletion, "delete")
        .tags([Tag::parse(vec!["e".to_owned(), "target".to_owned()]).expect("tag")])
        .sign_with_keys(keys)
        .expect("deletion")
}
