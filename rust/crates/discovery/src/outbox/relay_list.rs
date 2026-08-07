//! Parsing of NIP-65 write-relay declarations.

use crate::relay::url::normalize_relay_url;
use nostr_sdk::Event;
use std::collections::HashMap;

/// Validated write relays in first-seen order. The last declaration for
/// a URL decides whether it allows writes.
pub(super) fn write_urls(event: &Event) -> Vec<String> {
    let mut order = Vec::new();
    let mut writable = HashMap::new();
    for tag in event.tags.iter() {
        let Some((url, write)) = write_declaration(tag.as_slice()) else {
            continue;
        };
        if !writable.contains_key(&url) {
            order.push(url.clone());
        }
        writable.insert(url, write);
    }
    order.retain(|url| writable[url]);
    order
}

fn write_declaration(tag: &[String]) -> Option<(String, bool)> {
    if tag.len() < 2 || tag[0] != "r" {
        return None;
    }
    let url = normalize_relay_url(&tag[1])?;
    let write = tag.get(2).is_none_or(|marker| marker != "read");
    Some((url, write))
}
