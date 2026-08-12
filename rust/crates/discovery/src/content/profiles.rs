//! Creator identities from kind-0 metadata events. `display_name` wins
//! over `name`, NIP-01 replaceable-event ordering selects metadata, and a
//! creator without usable metadata receives an npub-based identity.

use std::collections::HashMap;

use nostr_sdk::{Event, EventId, Kind, PublicKey, Timestamp, ToBech32};
use serde_json::Value;
use url::Url;

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
    let name = fields.and_then(|profile| safe_handle(&profile.name, author));
    format!("@{}", name.as_deref().unwrap_or(npub))
}

/// `display_name` over `name`, skipping blanks; a name equal to the hex
/// public key is treated as missing.
fn display_name(fields: Option<&ProfileFields>, author: &PublicKey, npub: &str) -> String {
    let candidate = fields.and_then(|profile| {
        safe_name(&profile.display_name, author, 50)
            .or_else(|| safe_name(&profile.name, author, 50))
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
    Some(ProfileFields {
        created_at: event.created_at,
        event_id: event.id,
        display_name: string_field(&content, "display_name")?,
        name: string_field(&content, "name")?,
        picture: safe_picture(&string_field(&content, "picture")?),
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

fn safe_handle(value: &Option<String>, author: &PublicKey) -> Option<String> {
    let normalized = normalized_text(value)?;
    let without_at = normalized.trim_start_matches('@').trim_start();
    if without_at.eq_ignore_ascii_case(&author.to_hex()) {
        return None;
    }
    bounded_text(without_at, 30)
}

fn safe_name(value: &Option<String>, author: &PublicKey, maximum: usize) -> Option<String> {
    let normalized = normalized_text(value)?;
    if normalized.eq_ignore_ascii_case(&author.to_hex()) {
        return None;
    }
    bounded_text(&normalized, maximum)
}

fn bounded_text(value: &str, maximum: usize) -> Option<String> {
    let bounded: String = value.chars().take(maximum).collect();
    (!bounded.is_empty()).then_some(bounded)
}

fn normalized_text(value: &Option<String>) -> Option<String> {
    let mut output = String::new();
    let mut separator = false;
    for character in value.as_deref()?.chars() {
        if unsafe_text(character) {
            separator |= !output.is_empty();
        } else {
            if separator {
                output.push(' ');
                separator = false;
            }
            output.push(character);
        }
    }
    let trimmed = output.trim().to_owned();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn unsafe_text(character: char) -> bool {
    character <= '\u{20}'
        || ('\u{7f}'..='\u{9f}').contains(&character)
        || ('\u{202a}'..='\u{202e}').contains(&character)
        || ('\u{2066}'..='\u{2069}').contains(&character)
}

fn safe_picture(value: &Option<String>) -> Option<String> {
    let raw = value.as_deref()?.trim();
    if raw.len() > 2048 {
        return None;
    }
    let parsed = Url::parse(raw).ok()?;
    let safe_scheme = matches!(parsed.scheme(), "http" | "https");
    let safe_user = parsed.username().is_empty() && parsed.password().is_none();
    (safe_scheme && safe_user && parsed.host().is_some()).then(|| raw.to_owned())
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
