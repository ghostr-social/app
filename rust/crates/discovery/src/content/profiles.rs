//! Creator identities from kind-0 metadata events.
//!
//! `display_name` wins over `name`, NIP-01 replaceable-event ordering
//! selects metadata, and creators without usable metadata receive an
//! npub-based identity.

use std::collections::HashMap;

use nostr_sdk::{Event, EventId, Kind, PublicKey, Timestamp, ToBech32 as _};
use serde_json::Value;

mod sanitization;
use sanitization::{safe_handle, safe_name, safe_picture};

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
    event_id: EventId,
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
    /// not a JSON object are ignored. Newer timestamps win; equal-time
    /// events use the lowest event ID required by NIP-01.
    pub fn ingest(&mut self, event: &Event) {
        if event.kind != Kind::Metadata {
            return;
        }
        let Some(fields) = parsed_fields(event) else {
            return;
        };
        match self.profiles.get(&event.pubkey) {
            Some(current) if !is_newer(&fields, current) => {}
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
            handle: profile_handle(fields, author, &npub),
            avatar_url: fields.and_then(|profile| profile.picture.clone()),
        }
    }
}

fn profile_handle(fields: Option<&ProfileFields>, author: &PublicKey, npub: &str) -> String {
    let name = fields.and_then(|profile| safe_handle(profile.name.as_deref(), author));
    format!("@{}", name.as_deref().unwrap_or(npub))
}

/// `display_name` over `name`, skipping blanks; a name equal to the hex
/// public key is treated as missing.
fn display_name(fields: Option<&ProfileFields>, author: &PublicKey, npub: &str) -> String {
    let candidate = fields.and_then(|profile| {
        safe_name(profile.display_name.as_deref(), author, 50)
            .or_else(|| safe_name(profile.name.as_deref(), author, 50))
    });
    candidate.unwrap_or_else(|| short_npub(npub))
}

/// `'${npub.substring(0, 12)}…'` — the first twelve npub characters.
fn short_npub(npub: &str) -> String {
    let prefix: String = npub.chars().take(12).collect();
    format!("{prefix}…")
}

/// Blank content is an empty profile. Any other content must be a JSON
/// object with string-or-absent known fields, or the event is dropped.
fn parsed_fields(event: &Event) -> Option<ProfileFields> {
    if event.content.trim().is_empty() {
        return Some(empty_fields(event.created_at, event.id));
    }
    let content: Value = serde_json::from_str(&event.content).ok()?;
    content.is_object().then_some(())?;
    let display_name = string_field(&content, "display_name")?.into_value();
    let name = string_field(&content, "name")?.into_value();
    let picture = string_field(&content, "picture")?.into_value();
    Some(ProfileFields {
        created_at: event.created_at,
        event_id: event.id,
        display_name,
        name,
        picture: safe_picture(picture.as_deref()),
    })
}

fn empty_fields(created_at: Timestamp, event_id: EventId) -> ProfileFields {
    ProfileFields {
        created_at,
        event_id,
        display_name: None,
        name: None,
        picture: None,
    }
}

fn is_newer(candidate: &ProfileFields, current: &ProfileFields) -> bool {
    candidate.created_at > current.created_at
        || (candidate.created_at == current.created_at && candidate.event_id < current.event_id)
}

/// Returns absence for missing or null values and rejects non-string values.
fn string_field(content: &Value, key: &str) -> Option<StringField> {
    match content.get(key) {
        None | Some(Value::Null) => Some(StringField::Absent),
        Some(Value::String(value)) => Some(StringField::Present(value.clone())),
        Some(_) => None,
    }
}

enum StringField {
    Absent,
    Present(String),
}

impl StringField {
    fn into_value(self) -> Option<String> {
        match self {
            Self::Absent => None,
            Self::Present(value) => Some(value),
        }
    }
}
