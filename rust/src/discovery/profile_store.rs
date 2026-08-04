//! Creator identities from kind-0 metadata events. Parity sources:
//! lib/features/video_catalog/data/creator_profile_summary.dart (display
//! name, `@npub` handle, avatar, and the shortened-npub fallback), ndk's
//! `Metadata.getName`/`fromEvent` (display_name over name, whole event
//! dropped when the content casts fail), and the strictly-newer
//! replacement in lib/platform/nostr/ndk_nostr_profile_search.dart. A
//! creator with no stored metadata still gets a full fallback identity.

use std::collections::HashMap;

use nostr_sdk::{Event, Kind, PublicKey, Timestamp, ToBech32};
use serde_json::Value;

/// The creator identity every feed row renders.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorProfile {
    pub display_name: String,
    pub handle: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug)]
struct ProfileFields {
    created_at: Timestamp,
    display_name: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

/// Newest kind-0 metadata per creator.
#[derive(Debug, Default)]
pub struct ProfileStore {
    profiles: HashMap<PublicKey, ProfileFields>,
}

impl ProfileStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests one kind-0 event; other kinds and events whose content is
    /// not a JSON object are ignored, and only strictly newer metadata
    /// replaces what is stored (ties keep the existing profile).
    pub fn ingest(&mut self, event: &Event) {
        if event.kind != Kind::Metadata {
            return;
        }
        let Some(fields) = parsed_fields(event) else {
            return;
        };
        match self.profiles.get(&event.pubkey) {
            Some(current) if fields.created_at <= current.created_at => {}
            _ => {
                self.profiles.insert(event.pubkey, fields);
            }
        }
    }

    /// The identity shown for this creator, with the shortened-npub
    /// fallback when metadata is missing or nameless.
    pub fn profile(&self, author: &PublicKey) -> CreatorProfile {
        let npub = author.to_bech32().expect("public keys encode as npub");
        let fields = self.profiles.get(author);
        CreatorProfile {
            display_name: display_name(fields, author, &npub),
            handle: format!("@{npub}"),
            avatar_url: fields.and_then(|profile| profile.picture.clone()),
        }
    }
}

/// `display_name` over `name`, skipping blanks (`Metadata.getName`); a
/// name equal to the hex pubkey is no name (`creatorProfileSummary`).
fn display_name(fields: Option<&ProfileFields>, author: &PublicKey, npub: &str) -> String {
    let candidate = fields
        .and_then(|profile| non_blank(&profile.display_name).or(non_blank(&profile.name)));
    match candidate {
        Some(name) if name != author.to_hex() => name.to_owned(),
        _ => short_npub(npub),
    }
}

/// `'${npub.substring(0, 12)}…'` — the first twelve npub characters.
fn short_npub(npub: &str) -> String {
    let prefix: String = npub.chars().take(12).collect();
    format!("{prefix}…")
}

fn non_blank(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|name| !name.trim().is_empty())
}

/// Blank content is an empty profile (Dart skips parsing it); anything
/// else must be a JSON object with string-or-absent known fields, or the
/// whole event is dropped like ndk's throwing `as String?` casts.
fn parsed_fields(event: &Event) -> Option<ProfileFields> {
    if event.content.trim().is_empty() {
        return Some(empty_fields(event.created_at));
    }
    let content: Value = serde_json::from_str(&event.content).ok()?;
    content.is_object().then_some(())?;
    Some(ProfileFields {
        created_at: event.created_at,
        display_name: string_field(&content, "display_name")?,
        name: string_field(&content, "name")?,
        picture: string_field(&content, "picture")?,
    })
}

fn empty_fields(created_at: Timestamp) -> ProfileFields {
    ProfileFields {
        created_at,
        display_name: None,
        name: None,
        picture: None,
    }
}

/// `Some(None)` when absent or null, `None` (drop the event) when the
/// value is present but not a string.
fn string_field(content: &Value, key: &str) -> Option<Option<String>> {
    match content.get(key) {
        None | Some(Value::Null) => Some(None),
        Some(Value::String(value)) => Some(Some(value.clone())),
        Some(_) => None,
    }
}
