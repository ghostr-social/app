//! A signed-out main feed has no follows to route to. Its request stays
//! unscoped, asks for discovery relays, and uses configured read relays.

use nostr_sdk::Keys;
use ghostr_discovery::cache::ViewerScope;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::outbox::directory::OutboxDirectory;
use ghostr_discovery::query::search::{plan_discovery, OutboxLookup};
use ghostr_discovery::content::social_graph::SocialGraph;
use ghostr_discovery::query::video_filters::{DiscoveryFlow, DiscoveryRequest};

const BOOTSTRAP: &str = "wss://boot.example";

#[test]
fn feed_spec_signed_out_main_feed_queries_the_bootstrap_relays() {
    let graph = SocialGraph::new(Keys::generate().public_key());
    let request = FeedSpec::MainFeed { viewer: None }
        .page_request(None, &graph)
        .expect("main feeds always request a page");
    assert_eq!(
        request,
        DiscoveryRequest {
            viewer: ViewerScope::SignedOut,
            flow: DiscoveryFlow::Continuous,
            ..DiscoveryRequest::default()
        }
    );

    assert_eq!(
        plan_discovery(&request).outbox,
        OutboxLookup::DiscoveryRelays
    );

    let directory = OutboxDirectory::new(vec![BOOTSTRAP.to_owned()]);
    let relays = directory.discovery_relays(12);

    assert_eq!(relays, vec![BOOTSTRAP.to_owned()]);
}
