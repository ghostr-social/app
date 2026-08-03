use crate::video::event_identity::VIDEO_KINDS;
use crate::video::native_models::NativeEventIdentity;
use nostr_sdk::Event;
use std::collections::{HashMap, HashSet};

const MAX_DELETION_TAGS_SCANNED: usize = 128;
const MAX_DELETION_ADDRESS_BYTES: usize = 1_024;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ExactDeletion {
    author: String,
    event_id: String,
}

#[derive(Default)]
pub struct NativeDeletionTombstones {
    addresses: HashMap<String, u64>,
    capacity: usize,
    exact: HashMap<ExactDeletion, u64>,
}

impl NativeDeletionTombstones {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            ..Self::default()
        }
    }

    pub fn record(&mut self, event: &Event) {
        let author = event.pubkey.to_hex();
        let created_at = event.created_at.as_u64();
        for tag in event.tags.iter().take(MAX_DELETION_TAGS_SCANNED) {
            let values = tag.as_slice();
            let kind = values.first().map(String::as_str);
            if kind == Some("e") {
                self.record_exact(&author, values.get(1), created_at);
            }
            if kind == Some("a") {
                self.record_address(&author, values.get(1), created_at);
            }
        }
        self.trim();
    }

    pub fn deletes(&self, identity: &NativeEventIdentity, coordinate: &str) -> bool {
        let exact = ExactDeletion {
            author: identity.author_public_key_hex.clone(),
            event_id: identity.event_id.clone(),
        };
        self.exact.contains_key(&exact)
            || self
                .addresses
                .get(coordinate)
                .is_some_and(|deleted_through| identity.created_at <= *deleted_through)
    }

    fn record_exact(&mut self, author: &str, value: Option<&String>, created_at: u64) {
        let Some(event_id) = value.filter(|item| valid_event_id(item)) else {
            return;
        };
        let key = ExactDeletion {
            author: author.to_owned(),
            event_id: event_id.to_ascii_lowercase(),
        };
        self.exact
            .entry(key)
            .and_modify(|value| *value = (*value).max(created_at))
            .or_insert(created_at);
    }

    fn record_address(&mut self, author: &str, value: Option<&String>, created_at: u64) {
        let Some(address) = value.filter(|item| valid_address(item, author)) else {
            return;
        };
        self.addresses
            .entry(address.to_owned())
            .and_modify(|value| *value = (*value).max(created_at))
            .or_insert(created_at);
    }

    fn trim(&mut self) {
        trim_map(&mut self.addresses, self.capacity);
        if self.exact.len() > self.capacity {
            let retained = retained_keys(&self.exact, self.capacity);
            self.exact.retain(|key, _| retained.contains(key));
        }
    }
}

fn trim_map<K>(values: &mut HashMap<K, u64>, capacity: usize)
where
    K: Clone + Eq + std::hash::Hash + Ord,
{
    if values.len() > capacity {
        let retained = retained_keys(values, capacity);
        values.retain(|key, _| retained.contains(key));
    }
}

fn retained_keys<K>(values: &HashMap<K, u64>, capacity: usize) -> HashSet<K>
where
    K: Clone + Eq + std::hash::Hash + Ord,
{
    let mut keys = values.keys().cloned().collect::<Vec<_>>();
    keys.sort_by(|left, right| {
        values[right]
            .cmp(&values[left])
            .then_with(|| left.cmp(right))
    });
    keys.truncate(capacity);
    keys.into_iter().collect()
}

fn valid_event_id(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|item| item.is_ascii_hexdigit())
}

fn valid_address(value: &str, author: &str) -> bool {
    if value.len() > MAX_DELETION_ADDRESS_BYTES {
        return false;
    }
    let mut parts = value.splitn(3, ':');
    let kind = parts.next().and_then(|item| item.parse::<u16>().ok());
    let referenced_author = parts.next();
    let identifier = parts.next();
    kind.is_some_and(|item| item >= 30_000 && VIDEO_KINDS.contains(&item))
        && referenced_author == Some(author)
        && identifier.is_some_and(|item| !item.is_empty())
}
