//! Addressable coordinates are compared as published. ndk keys the
//! same-video coordinate on the raw `d` tag value
//! (`_eventCoordinate` in
//! lib/features/video_catalog/data/ndk_video_remote_source.dart, which
//! only rejects a blank one), so `"clip"` and `" clip "` are two
//! videos. Folding them together silently drops one of the two.

mod feed_support;

use feed_support::{addressable_video, parsed_posts};
use nostr_sdk::Keys;
use rust_lib_ghostr::discovery::feed_assembly::canonical_posts;

#[test]
fn feed_assembly_keeps_identifiers_that_differ_only_in_padding() {
    let keys = Keys::generate();
    let fetched = parsed_posts(&[
        addressable_video(&keys, "clip", "a", 40),
        addressable_video(&keys, " clip ", "b", 30),
    ]);

    let canonical = canonical_posts(fetched);

    assert_eq!(canonical.len(), 2);
}

/// The identifier Dart addresses social writes with stays trimmed, like
/// NostrEventIdentifier.parse.
#[test]
fn feed_assembly_still_reports_the_trimmed_identifier() {
    let keys = Keys::generate();
    let fetched = parsed_posts(&[addressable_video(&keys, " clip ", "a", 40)]);

    let canonical = canonical_posts(fetched);

    assert_eq!(canonical[0].identifier.as_deref(), Some("clip"));
}

#[test]
fn feed_assembly_still_folds_revisions_of_one_identifier() {
    let keys = Keys::generate();
    let fetched = parsed_posts(&[
        addressable_video(&keys, "clip", "a", 40),
        addressable_video(&keys, "clip", "b", 30),
    ]);

    let canonical = canonical_posts(fetched);

    assert_eq!(canonical.len(), 1);
    assert_eq!(canonical[0].created_at, 40);
}
