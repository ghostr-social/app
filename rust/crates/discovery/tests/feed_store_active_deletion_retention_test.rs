use crate::content::deletions::deletion_claims;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{
    empty_graph, parsed, signed_event, video_note, SignedEventFixture,
};
use nostr_sdk::{Event, Keys, Kind};

#[test]
fn unrelated_multi_author_claims_cannot_resurrect_retained_content() {
    let creator = Keys::generate();
    let original = video_note(&creator, "protected", 10);
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed { viewer: None });
    store.ingest_first_page(feed, vec![parsed(&original)], &empty_graph());
    store.ingest_deletions(
        feed,
        deletion_claims(&[deletion(&creator, vec![original.id.to_hex()], 20)]),
        &empty_graph(),
    );

    for created_at in 21..=28 {
        let attacker = Keys::generate();
        let targets = (0..500)
            .map(|index| format!("unrelated-{created_at}-{index}"))
            .collect();
        let claims = deletion_claims(&[deletion(&attacker, targets, created_at)]);
        store.ingest_deletions(feed, claims, &empty_graph());
    }

    assert!(store.posts(feed).is_empty());
}

fn deletion(keys: &Keys, targets: Vec<String>, created_at: u64) -> Event {
    let tags = targets
        .into_iter()
        .map(|target| vec!["e".to_owned(), target])
        .collect();
    signed_event(SignedEventFixture {
        keys,
        kind: Kind::EventDeletion,
        content: "delete",
        tags,
        created_at,
    })
}
