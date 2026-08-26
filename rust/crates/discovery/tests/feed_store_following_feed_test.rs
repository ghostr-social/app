//! A creator-scoped feed serves every creator it names. The Following
//! feed hands its whole follow set down
//! (`lib/features/video_catalog/domain/filtered_video_feed_repository.dart`
//! passes `followed` for FeedKind.following, and ndk turns it into the
//! query's `authors`), so a spec that kept only one of them would query
//! one author and then filter the page down to that same author —
//! everyone else's posts would vanish from the feed.

use crate::content::social_graph::SocialGraph;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{empty_graph, parsed_posts, video_note};
use nostr_sdk::Keys;

#[test]
fn feed_store_creator_feed_keeps_every_named_creators_posts() {
    let (first, second, stranger) = (Keys::generate(), Keys::generate(), Keys::generate());
    let graph = SocialGraph::new(Keys::generate().public_key());
    let mut store = FeedStore::new();
    let followed = vec![first.public_key(), second.public_key()];
    let feed = store.open_feed(FeedSpec::Profile(followed));

    let fetched = parsed_posts(&[
        video_note(&first, "one", 30),
        video_note(&second, "two", 25),
        video_note(&stranger, "stray", 20),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    let authors: Vec<&str> = store
        .posts(feed)
        .iter()
        .map(|post| post.author_pubkey.as_str())
        .collect();
    assert_eq!(
        authors,
        [
            first.public_key().to_hex().as_str(),
            second.public_key().to_hex().as_str()
        ]
    );
}

#[test]
fn feed_spec_creator_feed_asks_the_relays_for_every_creator() {
    let followed: Vec<_> = (0..3).map(|_| Keys::generate().public_key()).collect();

    let request = FeedSpec::Profile(followed.clone()).page_request(None, &empty_graph());

    assert_eq!(request.expect("request").authors, followed);
}
