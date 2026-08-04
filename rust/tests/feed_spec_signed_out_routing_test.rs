//! Signed out the main feed routes exactly like ndk's: no viewer means
//! no follows to route to, so the request stays unscoped, its outbox
//! lookup asks for discovery relays, and with no follow relay lists
//! known the directory answers with the bootstrap set alone — mirrors
//! `_followedPubkeys` returning nothing and `_merged` yielding the
//! bootstrap urls in lib/platform/nostr/ndk_nostr_outbox_directory.dart.

use rust_lib_ghostr::discovery::feed_spec::FeedSpec;
use rust_lib_ghostr::discovery::outbox_directory::OutboxDirectory;
use rust_lib_ghostr::discovery::search_queries::{plan_discovery, OutboxLookup};
use rust_lib_ghostr::discovery::video_filters::DiscoveryRequest;

const BOOTSTRAP: &str = "wss://boot.example";

#[test]
fn feed_spec_signed_out_main_feed_queries_the_bootstrap_relays() {
    let request = FeedSpec::MainFeed { viewer: None }
        .page_request(None)
        .expect("main feeds always request a page");
    assert_eq!(request, DiscoveryRequest::default());

    assert_eq!(plan_discovery(&request).outbox, OutboxLookup::DiscoveryRelays);

    let directory = OutboxDirectory::new(vec![BOOTSTRAP.to_owned()]);
    let relays = directory.relays_for_authors(&[], 12);

    assert_eq!(relays, vec![BOOTSTRAP.to_owned()]);
}
