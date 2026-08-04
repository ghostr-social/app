//! Hashtag case-variant expansion for `#t` tag filters. Parity source:
//! lib/features/video_catalog/domain/video_hashtags.dart — its variant
//! set is a product decision this module must match exactly.

/// Lowercased, `#`-stripped form of a hashtag; `None` when nothing remains.
pub fn normalize_hashtag(raw: &str) -> Option<String> {
    let lowered = raw.trim().to_lowercase();
    let value = lowered.strip_prefix('#').unwrap_or(&lowered);
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Relays match tag values exactly, so a hashtag query must ask for every
/// case form publishers commonly write: as-typed, lower, UPPER, and Title,
/// deduplicated in that order.
pub fn hashtag_query_variants(raw: &str) -> Vec<String> {
    let typed = strip_hash(raw.trim());
    match normalize_hashtag(typed) {
        None => Vec::new(),
        Some(tag) => {
            let upper = tag.to_uppercase();
            let title = title_case(&tag);
            dedup_in_order(vec![typed.to_string(), tag, upper, title])
        }
    }
}

/// Expanded, deduplicated `#t` values for a whole hashtag set
/// (video_discovery_queries.dart `_hashtagFilters`).
pub fn hashtag_filter_values(hashtags: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    for hashtag in hashtags {
        values.extend(hashtag_query_variants(hashtag));
    }
    dedup_in_order(values)
}

fn strip_hash(value: &str) -> &str {
    value.strip_prefix('#').unwrap_or(value)
}

/// Dart uppercases the first UTF-16 unit; uppercasing the first `char` is
/// the sane equivalent for the tags this app meets.
fn title_case(tag: &str) -> String {
    let mut chars = tag.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

fn dedup_in_order(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::with_capacity(values.len());
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}
