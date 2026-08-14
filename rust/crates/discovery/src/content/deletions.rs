//! Author-valid NIP-09 deletion claims applied to feed occurrences.

pub use super::deletion_index::DeletionIndex;
use nostr_sdk::{Event, Kind};

const DELETION_TARGETS_PER_EVENT: usize = 500;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeletionClaim {
    pub(super) target: DeletionTarget,
    pub(super) deleter_pubkey: String,
    pub(super) deleted_at: u64,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum DeletionTarget {
    Event(String),
    Address(String),
}

pub fn deletion_claims(events: &[Event]) -> Vec<DeletionClaim> {
    events.iter().flat_map(claims_for_event).collect()
}

fn claims_for_event(event: &Event) -> Vec<DeletionClaim> {
    if event.kind != Kind::EventDeletion || event.verify().is_err() {
        return Vec::new();
    }
    let deleter = event.pubkey.to_hex();
    event
        .tags
        .iter()
        .filter_map(|tag| deletion_target(tag.as_slice(), &deleter))
        .take(DELETION_TARGETS_PER_EVENT)
        .map(|target| DeletionClaim {
            target,
            deleter_pubkey: deleter.clone(),
            deleted_at: event.created_at.as_u64(),
        })
        .collect()
}

fn deletion_target(tag: &[String], deleter: &str) -> Option<DeletionTarget> {
    match tag.first().map(String::as_str) {
        Some("e") => tag.get(1).cloned().map(DeletionTarget::Event),
        Some("a") => address_target(tag.get(1)?, deleter),
        _ => None,
    }
}

fn address_target(value: &str, deleter: &str) -> Option<DeletionTarget> {
    let mut parts = value.splitn(3, ':');
    parts.next()?.parse::<u16>().ok()?;
    let author = parts.next()?;
    let identifier = parts.next()?;
    if author != deleter || identifier.trim().is_empty() {
        return None;
    }
    Some(DeletionTarget::Address(value.to_owned()))
}
