use crate::video::native_media_metadata::native_media;
use crate::video::native_models::{NativeEventIdentity, NativeUserData, NativeVideo};
use crate::video::native_text::{bounded_native_text, MAX_NATIVE_IDENTIFIER_BYTES};
use nostr_sdk::{Event, ToBech32};
use sha2::{Digest, Sha256};

pub const VIDEO_KINDS: [u16; 4] = [21, 22, 34235, 34236];
pub const MAX_NATIVE_MEDIA_PER_EVENT: usize = 5;

#[derive(Clone)]
pub struct CanonicalNativeVideo {
    pub inventory_id: String,
    pub coordinate: String,
    pub identity: NativeEventIdentity,
    pub video: NativeVideo,
}

#[derive(Clone)]
pub(crate) struct CanonicalEvent {
    pub coordinate: String,
    pub identity: NativeEventIdentity,
}

pub fn canonical_video_events(event: &Event) -> Vec<(String, NativeEventIdentity)> {
    canonical_native_videos(event)
        .into_iter()
        .map(|item| (item.video.id, item.identity))
        .collect()
}

pub fn canonical_native_videos(event: &Event) -> Vec<CanonicalNativeVideo> {
    let Some(canonical) = canonical_event(event) else {
        return Vec::new();
    };
    event
        .tags
        .iter()
        .filter_map(|tag| native_video(event, tag.as_slice()))
        .take(MAX_NATIVE_MEDIA_PER_EVENT)
        .map(|video| CanonicalNativeVideo {
            inventory_id: inventory_id(&canonical.coordinate, &canonical.identity.event_id, &video),
            coordinate: canonical.coordinate.clone(),
            identity: canonical.identity.clone(),
            video,
        })
        .collect()
}

pub(crate) fn canonical_event(event: &Event) -> Option<CanonicalEvent> {
    let kind = event.kind.as_u16();
    if !VIDEO_KINDS.contains(&kind) {
        return None;
    }
    let identifier = event_identifier(event, kind);
    if kind >= 30_000 && identifier.is_none() {
        return None;
    }
    Some(CanonicalEvent {
        coordinate: event_coordinate(event, kind, identifier.as_deref()),
        identity: event_identity(event, identifier),
    })
}

fn event_identity(event: &Event, identifier: Option<String>) -> NativeEventIdentity {
    NativeEventIdentity {
        event_id: event.id.to_hex(),
        author_public_key_hex: event.pubkey.to_hex(),
        kind: event.kind.as_u16(),
        identifier,
        created_at: event.created_at.as_u64(),
        content: bounded_native_text(&event.content),
    }
}

fn native_video(event: &Event, tag: &[String]) -> Option<NativeVideo> {
    let media = native_media(tag)?;
    let id = media
        .expected_digest
        .clone()
        .unwrap_or_else(|| hashless_cache_id(event, &media.url));
    Some(NativeVideo {
        id,
        expected_digest: media.expected_digest,
        fallback_urls: media.fallback_urls,
        user: NativeUserData {
            npub: event.pubkey.to_bech32().ok(),
            name: None,
            profile_picture: None,
        },
        title: bounded_native_text(media.title.as_deref().unwrap_or(&event.content)),
        song_name: tag_value(event, "title")
            .map(bounded_native_text)
            .unwrap_or_else(|| "Original sound".to_owned()),
        comments: "0".to_owned(),
        likes: "0".to_owned(),
        url: media.url,
        delivery: media.delivery,
    })
}

fn hashless_cache_id(event: &Event, url: &str) -> String {
    sha256(&format!("{}\0{url}", event.id.to_hex()))
}

fn event_coordinate(event: &Event, kind: u16, identifier: Option<&str>) -> String {
    match identifier {
        Some(identifier) => format!("{kind}:{}:{identifier}", event.pubkey.to_hex()),
        None => event.id.to_hex(),
    }
}

fn inventory_id(coordinate: &str, event_id: &str, video: &NativeVideo) -> String {
    sha256(&format!(
        "{coordinate}\0{event_id}\0{}\0{}",
        video.id, video.url
    ))
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn event_identifier(event: &Event, kind: u16) -> Option<String> {
    if kind < 30_000 {
        return None;
    }
    let identifier = tag_value(event, "d")?;
    (identifier.len() <= MAX_NATIVE_IDENTIFIER_BYTES).then(|| identifier.to_owned())
}

fn tag_value<'a>(event: &'a Event, name: &str) -> Option<&'a str> {
    event.tags.iter().find_map(|tag| {
        let values = tag.as_slice();
        (values.first().map(String::as_str) == Some(name))
            .then(|| values.get(1).map(String::as_str))
            .flatten()
            .filter(|value| !value.trim().is_empty())
    })
}
