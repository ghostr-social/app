//! A signed-out main feed filters nothing: with no viewer there is no
//! social graph whose mutes could apply, so the page stays the unscoped
//! global page. Viewer-scoped feeds still drop muted creators.

mod discovery_support;
mod feed_support;

use discovery_support::{mute_list, p_tag};
use feed_support::{parsed_posts, video_note};
use nostr_sdk::Keys;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use ghostr_discovery::content::social_graph::SocialGraph;

#[test]
fn feed_store_signed_out_main_feed_keeps_every_creator() {
    let (session, muted) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());
    graph.ingest(&mute_list(&session, vec![p_tag(&muted.public_key())], 5));
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed { viewer: None });

    let fetched = parsed_posts(&[video_note(&muted, "spam", 30)]);
    store.ingest_first_page(feed, fetched, &graph);

    let authors: Vec<&str> = store
        .posts(feed)
        .iter()
        .map(|post| post.author_pubkey.as_str())
        .collect();
    assert_eq!(authors, [muted.public_key().to_hex().as_str()]);
}
