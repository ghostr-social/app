use super::*;

/// One canonical post per coordinate — newest `created_at` wins, ties keep
/// the lexicographically smaller event id — ordered newest-first with
/// ascending-ID tiebreak.
pub(crate) fn canonical_posts(fetched: Vec<ParsedVideoPost>) -> Vec<ParsedVideoPost> {
    canonical_posts_from_axes(fetched.clone(), fetched)
}
