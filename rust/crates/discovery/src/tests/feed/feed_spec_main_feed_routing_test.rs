//! The main feed routes by the viewer's follows without filtering by
//! them: `page_request` puts the follows in `routing_authors`, never in
//! `authors`. The unscoped query is outbox-routed, so the follows' relays
//! answer with everything they carry.

use crate::feed::spec::FeedSpec;
use crate::content::social_graph::SocialGraph;
use crate::tests::outbox_support::contact_list_event;
use nostr_sdk::prelude::*;

fn graph_following(viewer: &Keys, follows: &[PublicKey]) -> SocialGraph {
    let mut graph = SocialGraph::new(viewer.public_key());
    graph.ingest(&contact_list_event(viewer, follows));
    graph
}

#[test]
fn the_main_feed_routes_by_follows_and_filters_by_nobody() {
    let viewer = Keys::generate();
    let follow = Keys::generate().public_key();
    let graph = graph_following(&viewer, &[follow]);

    let request = FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    }
    .page_request(None, &graph)
    .expect("the main feed always queries");

    assert_eq!(request.routing_authors, vec![follow]);
    assert!(
        request.authors.is_empty(),
        "the main feed filters by nobody"
    );
}

/// Signed out there is no graph to route by, so the request stays the
/// unscoped global page the bootstrap relays answer.
#[test]
fn a_signed_out_main_feed_routes_by_nobody() {
    let graph = SocialGraph::new(Keys::generate().public_key());

    let request = FeedSpec::MainFeed { viewer: None }
        .page_request(None, &graph)
        .expect("the main feed always queries");

    assert!(request.routing_authors.is_empty());
}

/// Profile grids already name their creators; nothing routing-only is
/// added on top.
#[test]
fn a_profile_feed_routes_by_the_creators_it_filters_by() {
    let viewer = Keys::generate();
    let creator = Keys::generate().public_key();
    let graph = graph_following(&viewer, &[Keys::generate().public_key()]);

    let request = FeedSpec::Profile(vec![creator])
        .page_request(None, &graph)
        .expect("profile feeds always query");

    assert_eq!(request.authors, vec![creator]);
    assert!(request.routing_authors.is_empty());
}
