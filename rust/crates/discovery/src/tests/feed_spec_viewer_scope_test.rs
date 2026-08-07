//! Which feed tells the session pool whose events it holds. Only the
//! main feed knows the viewer, so only it scopes the pool; a search,
//! hashtag or profile feed names nobody and leaves the scope alone.

use crate::event_cache::ViewerScope;
use crate::feed_spec::FeedSpec;
use crate::social_graph::SocialGraph;
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};

fn scope(spec: &FeedSpec) -> ViewerScope {
    let graph = SocialGraph::new(author(AUTHOR_B));
    spec.page_request(None, &graph)
        .expect("the spec produces a request")
        .viewer
}

#[test]
fn a_signed_in_main_feed_scopes_the_pool_to_its_viewer() {
    let spec = FeedSpec::MainFeed {
        viewer: Some(author(AUTHOR_A)),
    };

    assert_eq!(scope(&spec), ViewerScope::SignedIn(author(AUTHOR_A)));
}

#[test]
fn a_signed_out_main_feed_scopes_the_pool_to_nobody() {
    assert_eq!(
        scope(&FeedSpec::MainFeed { viewer: None }),
        ViewerScope::SignedOut
    );
}

#[test]
fn a_query_feed_leaves_the_scope_alone() {
    assert_eq!(
        scope(&FeedSpec::Search("skate".into())),
        ViewerScope::Unknown
    );
    assert_eq!(
        scope(&FeedSpec::Hashtag("skate".into())),
        ViewerScope::Unknown
    );
    assert_eq!(
        scope(&FeedSpec::Profile(vec![author(AUTHOR_A)])),
        ViewerScope::Unknown
    );
}
