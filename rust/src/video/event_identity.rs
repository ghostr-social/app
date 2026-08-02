use crate::video::native_models::{NativeUserData, NativeVideo, NativeVideoDelivery};
use crate::video::video::FfiNostrEventIdentity;
use nostr_sdk::{Event, ToBech32};
use sha2::{Digest, Sha256};

pub const VIDEO_KINDS: [u16; 4] = [21, 22, 34235, 34236];

#[derive(Clone)]
pub struct CanonicalNativeVideo {
    pub inventory_id: String,
    pub coordinate: String,
    pub identity: FfiNostrEventIdentity,
    pub video: NativeVideo,
}

struct CanonicalEvent {
    coordinate: String,
    identity: FfiNostrEventIdentity,
}

struct NativeMediaMetadata {
    cache_id: String,
    delivery: NativeVideoDelivery,
    title: Option<String>,
    url: String,
}

pub fn canonical_video_events(event: &Event) -> Vec<(String, FfiNostrEventIdentity)> {
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
        .map(|video| CanonicalNativeVideo {
            inventory_id: inventory_id(&canonical.coordinate, &video),
            coordinate: canonical.coordinate.clone(),
            identity: canonical.identity.clone(),
            video,
        })
        .collect()
}

fn canonical_event(event: &Event) -> Option<CanonicalEvent> {
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

fn event_identity(event: &Event, identifier: Option<String>) -> FfiNostrEventIdentity {
    FfiNostrEventIdentity {
        event_id: event.id.to_hex(),
        author_public_key_hex: event.pubkey.to_hex(),
        kind: event.kind.as_u16() as u64,
        identifier,
        created_at: event.created_at.as_u64(),
        content: event.content.clone(),
    }
}

fn native_video(event: &Event, tag: &[String]) -> Option<NativeVideo> {
    let media = native_media(tag)?;
    Some(NativeVideo {
        id: media.cache_id,
        user: NativeUserData {
            npub: event.pubkey.to_bech32().ok(),
            name: None,
            profile_picture: None,
        },
        title: media.title.unwrap_or_else(|| event.content.clone()),
        song_name: tag_value(event, "title").unwrap_or_else(|| "Original sound".to_owned()),
        comments: "0".to_owned(),
        likes: "0".to_owned(),
        url: media.url,
        delivery: media.delivery,
    })
}

fn native_media(tag: &[String]) -> Option<NativeMediaMetadata> {
    if tag.first().map(String::as_str) != Some("imeta") {
        return None;
    }
    let mime = imeta_field(tag, "m")?;
    if !is_video_mime(&mime) {
        return None;
    }
    let url = imeta_field(tag, "url")?;
    if !is_http_url(&url) {
        return None;
    }
    Some(NativeMediaMetadata {
        cache_id: video_cache_key(tag, &url)?,
        delivery: media_delivery(&mime),
        title: imeta_field(tag, "title"),
        url,
    })
}

fn video_cache_key(tag: &[String], url: &str) -> Option<String> {
    let Some(digest) = imeta_field(tag, "x") else {
        return Some(sha256(url));
    };
    (digest.len() == 64 && digest.chars().all(|value| value.is_ascii_hexdigit()))
        .then(|| digest.to_ascii_lowercase())
}

fn is_video_mime(value: &str) -> bool {
    value.starts_with("video/") || value.eq_ignore_ascii_case("application/x-mpegurl")
}

fn media_delivery(value: &str) -> NativeVideoDelivery {
    if value.eq_ignore_ascii_case("application/x-mpegurl") {
        NativeVideoDelivery::Hls
    } else {
        NativeVideoDelivery::Progressive
    }
}

fn event_coordinate(event: &Event, kind: u16, identifier: Option<&str>) -> String {
    match identifier {
        Some(identifier) => format!("{kind}:{}:{identifier}", event.pubkey.to_hex()),
        None => event.id.to_hex(),
    }
}

fn inventory_id(coordinate: &str, video: &NativeVideo) -> String {
    sha256(&format!("{coordinate}\0{}\0{}", video.id, video.url))
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn imeta_field(tag: &[String], name: &str) -> Option<String> {
    tag.iter().skip(1).find_map(|value| {
        let (key, field) = value.split_once(char::is_whitespace)?;
        (key == name && !field.trim().is_empty()).then(|| field.trim().to_owned())
    })
}

fn event_identifier(event: &Event, kind: u16) -> Option<String> {
    (kind >= 30_000).then(|| tag_value(event, "d")).flatten()
}

fn tag_value(event: &Event, name: &str) -> Option<String> {
    event.tags.iter().find_map(|tag| {
        let values = tag.as_slice();
        (values.first().map(String::as_str) == Some(name))
            .then(|| values.get(1).cloned())
            .flatten()
            .filter(|value| !value.trim().is_empty())
    })
}

fn is_http_url(value: &str) -> bool {
    reqwest::Url::parse(value)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some())
        .unwrap_or(false)
}
