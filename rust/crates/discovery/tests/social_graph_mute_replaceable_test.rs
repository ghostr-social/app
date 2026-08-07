//! Kind-10000 mute lists are replaceable: strictly newer created_at
//! wins, an equally old or older list is ignored, and an unmute takes
//! effect through the newer list — mirrors `_newestMute` in
//! lib/platform/nostr/ndk_nostr_social_models.dart.

mod discovery_support;

use discovery_support::{mute_list, p_tag};
use nostr_sdk::Keys;
use ghostr_discovery::content::social_graph::SocialGraph;

#[test]
fn newer_mute_list_unmutes_removed_creators() {
    let session = Keys::generate();
    let (muted, forgiven) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&mute_list(
        &session,
        vec![p_tag(&muted.public_key()), p_tag(&forgiven.public_key())],
        10,
    ));
    graph.ingest(&mute_list(&session, vec![p_tag(&muted.public_key())], 20));

    assert!(graph.is_muted(&muted.public_key()));
    assert!(!graph.is_muted(&forgiven.public_key()));
}

#[test]
fn stale_mute_list_arriving_late_is_ignored() {
    let session = Keys::generate();
    let stale_target = Keys::generate();
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&mute_list(&session, Vec::new(), 20));
    graph.ingest(&mute_list(
        &session,
        vec![p_tag(&stale_target.public_key())],
        10,
    ));

    assert!(!graph.is_muted(&stale_target.public_key()));
}

#[test]
fn equal_created_at_keeps_the_existing_mute_list() {
    let session = Keys::generate();
    let (first, second) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&mute_list(&session, vec![p_tag(&first.public_key())], 10));
    graph.ingest(&mute_list(&session, vec![p_tag(&second.public_key())], 10));

    assert!(graph.is_muted(&first.public_key()));
    assert!(!graph.is_muted(&second.public_key()));
}
