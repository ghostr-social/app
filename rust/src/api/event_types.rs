//! Typed FFI shapes for generic Nostr reads.

use anyhow::{Context as _, Result};
use nostr_sdk::{Event, EventId, Filter, Kind, PublicKey, SingleLetterTag, Timestamp};

/// One case-sensitive single-letter Nostr tag filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiNostrTagFilter {
    pub name: String,
    pub values: Vec<String>,
}

/// One Nostr REQ filter supplied by Dart.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiNostrEventFilter {
    pub kinds: Vec<u16>,
    pub authors: Vec<String>,
    pub event_tags: Vec<String>,
    pub tag_filters: Vec<FfiNostrTagFilter>,
    pub limit: u32,
    pub until: Option<u64>,
    pub search: Option<String>,
}

/// One verified Nostr event returned to Dart.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiNostrEvent {
    pub id: String,
    pub pubkey: String,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub created_at: u64,
}

impl TryFrom<FfiNostrEventFilter> for Filter {
    type Error = anyhow::Error;

    fn try_from(value: FfiNostrEventFilter) -> Result<Self> {
        let FfiNostrEventFilter {
            kinds,
            authors,
            event_tags,
            tag_filters,
            limit,
            until,
            search,
        } = value;
        anyhow::ensure!(limit > 0, "the query limit must be positive");
        let mut filter = Self::new()
            .kinds(kinds.into_iter().map(Kind::from))
            .limit(limit as usize);
        filter = with_authors(filter, authors)?;
        filter = with_event_tags(filter, event_tags)?;
        filter = with_generic_tags(filter, tag_filters)?;
        filter = with_until(filter, until);
        Ok(with_search(filter, search))
    }
}

impl From<&Event> for FfiNostrEvent {
    fn from(event: &Event) -> Self {
        Self {
            id: event.id.to_hex(),
            pubkey: event.pubkey.to_hex(),
            kind: event.kind.as_u16(),
            tags: event
                .tags
                .iter()
                .map(|tag| tag.as_slice().to_vec())
                .collect(),
            content: event.content.clone(),
            created_at: event.created_at.as_u64(),
        }
    }
}

fn with_authors(mut filter: Filter, raw: Vec<String>) -> Result<Filter> {
    for author in raw {
        let author = PublicKey::from_hex(&author).context("invalid query author")?;
        filter = filter.author(author);
    }
    Ok(filter)
}

fn with_event_tags(mut filter: Filter, raw: Vec<String>) -> Result<Filter> {
    for event in raw {
        let event = EventId::from_hex(&event).context("invalid query event id")?;
        filter = filter.event(event);
    }
    Ok(filter)
}

fn with_generic_tags(mut filter: Filter, raw: Vec<FfiNostrTagFilter>) -> Result<Filter> {
    for tag in raw {
        let letter = tag_letter(&tag.name)?;
        filter = filter.custom_tag(letter, tag.values);
    }
    Ok(filter)
}

fn tag_letter(name: &str) -> Result<SingleLetterTag> {
    let mut chars = name.chars();
    let letter = chars.next().context("query tag names must be one letter")?;
    anyhow::ensure!(chars.next().is_none(), "query tag names must be one letter");
    SingleLetterTag::from_char(letter).context("query tag names must be ASCII letters")
}

fn with_until(filter: Filter, until: Option<u64>) -> Filter {
    match until {
        Some(until) => filter.until(Timestamp::from(until)),
        None => filter,
    }
}

fn with_search(filter: Filter, search: Option<String>) -> Filter {
    match search {
        Some(search) => filter.search(search),
        None => filter,
    }
}
