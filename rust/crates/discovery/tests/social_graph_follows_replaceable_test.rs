//! Kind-3 contact lists are replaceable: strictly newer created_at
//! wins and an equally old or older list is ignored — mirrors
//! `_newestContact` in lib/platform/nostr/ndk_nostr_social_models.dart.

mod discovery_support;

use discovery_support::{contact_list, p_tag};
use nostr_sdk::Keys;
use ghostr_discovery::social_graph::SocialGraph;

#[test]
fn newer_contact_list_replaces_the_previous_one() {
    let session = Keys::generate();
    let (old_follow, new_follow) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&contact_list(
        &session,
        vec![p_tag(&old_follow.public_key())],
        10,
    ));
    graph.ingest(&contact_list(
        &session,
        vec![p_tag(&new_follow.public_key())],
        20,
    ));

    assert!(graph.follows().contains(&new_follow.public_key()));
    assert!(!graph.follows().contains(&old_follow.public_key()));
}

#[test]
fn stale_contact_list_arriving_late_is_ignored() {
    let session = Keys::generate();
    let (old_follow, new_follow) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&contact_list(
        &session,
        vec![p_tag(&new_follow.public_key())],
        20,
    ));
    graph.ingest(&contact_list(
        &session,
        vec![p_tag(&old_follow.public_key())],
        10,
    ));

    assert!(graph.follows().contains(&new_follow.public_key()));
    assert!(!graph.follows().contains(&old_follow.public_key()));
}

#[test]
fn equal_created_at_keeps_the_existing_contact_list() {
    let session = Keys::generate();
    let (first, second) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&contact_list(
        &session,
        vec![p_tag(&first.public_key())],
        10,
    ));
    graph.ingest(&contact_list(
        &session,
        vec![p_tag(&second.public_key())],
        10,
    ));

    assert!(graph.follows().contains(&first.public_key()));
    assert!(!graph.follows().contains(&second.public_key()));
}
