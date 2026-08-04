//! The session's kind-3 contact list yields the followed pubkey set:
//! p tags only, deduplicated, other tag names and other authors'
//! contact lists ignored — mirrors `_loadFollowedProfiles` in
//! lib/platform/nostr/ndk_nostr_social.dart (contacts to a Set).

mod discovery_support;

use discovery_support::{contact_list, p_tag};
use nostr_sdk::Keys;
use rust_lib_ghostr::discovery::social_graph::SocialGraph;

#[test]
fn collects_deduplicated_p_tag_follows() {
    let session = Keys::generate();
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&contact_list(
        &session,
        vec![
            p_tag(&alice.public_key()),
            p_tag(&bob.public_key()),
            p_tag(&alice.public_key()),
            vec!["t".to_owned(), "video".to_owned()],
            vec!["p".to_owned(), "not-a-pubkey".to_owned()],
        ],
        10,
    ));

    assert_eq!(graph.follows().len(), 2);
    assert!(graph.follows().contains(&alice.public_key()));
    assert!(graph.follows().contains(&bob.public_key()));
}

#[test]
fn ignores_contact_lists_from_other_authors() {
    let session = Keys::generate();
    let stranger = Keys::generate();
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&contact_list(
        &stranger,
        vec![p_tag(&Keys::generate().public_key())],
        10,
    ));

    assert!(graph.follows().is_empty());
}

#[test]
fn starts_with_no_follows() {
    let graph = SocialGraph::new(Keys::generate().public_key());

    assert!(graph.follows().is_empty());
}
