//! A profile feed shows exactly the target creator's posts: strangers'
//! posts are dropped, and even a muted target keeps their own grid —
//! mirrors `ProfileDetailsPolicy.build` filtering only by creator id in
//! lib/features/video_catalog/domain/profile_details_policy.dart (a
//! blocked creator's page still lists their posts; only the
//! relationship flags the block).

mod discovery_support;
mod feed_support;

use discovery_support::{mute_list, p_tag};
use feed_support::{parsed_posts, video_note};
use nostr_sdk::Keys;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use ghostr_discovery::content::social_graph::SocialGraph;

#[test]
fn feed_store_profile_feed_keeps_only_the_target_creators_posts() {
    let (target, stranger) = (Keys::generate(), Keys::generate());
    let graph = SocialGraph::new(Keys::generate().public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Profile(vec![target.public_key()]));

    let fetched = parsed_posts(&[
        video_note(&target, "own", 30),
        video_note(&stranger, "stray", 20),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    let authors: Vec<&str> = store
        .posts(feed)
        .iter()
        .map(|post| post.author_pubkey.as_str())
        .collect();
    assert_eq!(authors, [target.public_key().to_hex().as_str()]);
}

#[test]
fn feed_store_profile_feed_shows_a_muted_targets_own_posts() {
    let session = Keys::generate();
    let target = Keys::generate();
    let mut graph = SocialGraph::new(session.public_key());
    graph.ingest(&mute_list(&session, vec![p_tag(&target.public_key())], 5));
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Profile(vec![target.public_key()]));

    store.ingest_first_page(
        feed,
        parsed_posts(&[video_note(&target, "own", 30)]),
        &graph,
    );

    assert_eq!(store.posts(feed).len(), 1);
}
