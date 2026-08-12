//! Addressable coordinates compare the raw published `d` value, so
//! `"clip"` and `" clip "` name distinct videos. Folding them together
//! would silently drop one.

mod feed_support;

use feed_support::{addressable_video, parsed_posts};
use ghostr_discovery::feed::assembly::canonical_posts;
use nostr_sdk::Keys;

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

/// The domain identifier used for social writes remains trimmed.
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
