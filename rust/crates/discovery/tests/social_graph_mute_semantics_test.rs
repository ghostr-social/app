//! Mutes hide creators, not hashtags or single events: only the mute
//! list's p tags mute, and a muted creator's posts are filtered by
//! author — mirrors `_loadBlockedProfiles` reading only `pubKeys` in
//! `lib/platform/nostr/ndk_nostr_social.dart` and the blocked-creator
//! filter in `lib/features/video_catalog/domain/video_feed_policy.dart`.

use crate::content::social_graph::SocialGraph;
use crate::tests::discovery_support::{mute_list, p_tag, plain_note};
use nostr_sdk::Keys;

#[test]
fn mutes_creators_listed_in_p_tags_only() {
    let session = Keys::generate();
    let muted = Keys::generate();
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&mute_list(
        &session,
        vec![
            p_tag(&muted.public_key()),
            vec!["t".to_owned(), "spamtag".to_owned()],
            vec!["word".to_owned(), "spamword".to_owned()],
            vec!["e".to_owned(), "a".repeat(64)],
        ],
        10,
    ));

    assert!(graph.is_muted(&muted.public_key()));
    assert!(!graph.is_muted(&session.public_key()));
}

#[test]
fn filters_posts_authored_by_muted_creators() {
    let session = Keys::generate();
    let (muted, visible) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());
    graph.ingest(&mute_list(&session, vec![p_tag(&muted.public_key())], 10));

    assert!(graph.is_muted(&plain_note(&muted, 20).pubkey));
    assert!(!graph.is_muted(&plain_note(&visible, 20).pubkey));
}

#[test]
fn ignores_mute_lists_from_other_authors() {
    let session = Keys::generate();
    let (stranger, target) = (Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());

    graph.ingest(&mute_list(&stranger, vec![p_tag(&target.public_key())], 10));

    assert!(!graph.is_muted(&target.public_key()));
}
