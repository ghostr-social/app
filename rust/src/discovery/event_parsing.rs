//! One Nostr event -> one playable video post. The Dart pipeline is the
//! specification: media per lib/features/video_catalog/data/
//! nostr_video_media.dart, display fields per nostr_video_event_mapper.dart,
//! accepted kinds per lib/platform/nostr/video_discovery_queries.dart.
//! Feed assembly consumes [`video_post_from_event`]; nothing here does IO.

use crate::engine::{DeliveryKind, VideoMeta};
use crate::video::event_identity::VIDEO_KINDS;
use crate::video::native_media_metadata::NativeMediaMetadata;
use crate::video::native_models::NativeVideoDelivery;
use crate::video::nostr_event_media::{event_media, tag_values};
use crate::video::post_text::{
    caption_without_urls, content_hashtags, normalized_hashtag, push_unique,
};
use nostr_sdk::Event;

pub const NOTE_KIND: u16 = 1;
pub const FILE_METADATA_KIND: u16 = 1063;

/// Server-side `#m` filter on the kind-1063 discovery query
/// (video_discovery_queries.dart `videoFileMimeTypes`, matched exactly).
const VIDEO_FILE_MIME_TYPES: [&str; 6] = [
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/mpeg",
    "application/x-mpegurl",
    "application/vnd.apple.mpegurl",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedVideoPost {
    pub event_id: String,
    pub author_pubkey: String,
    pub kind: u16,
    pub identifier: Option<String>,
    pub created_at: u64,
    pub caption: String,
    pub title: Option<String>,
    pub hashtags: Vec<String>,
    pub dimensions: Option<(u32, u32)>,
    pub blurhash: Option<String>,
    pub thumbnail_url: Option<String>,
    pub meta: VideoMeta,
}

pub fn video_post_from_event(event: &Event) -> Option<ParsedVideoPost> {
    if !accepts_kind(event) {
        return None;
    }
    let identifier = addressable_identifier(event)?;
    let media = event_media(event)?;
    Some(parsed_post(event, identifier, media))
}

fn accepts_kind(event: &Event) -> bool {
    let kind = event.kind.as_u16();
    if kind == NOTE_KIND || VIDEO_KINDS.contains(&kind) {
        return true;
    }
    kind == FILE_METADATA_KIND && has_video_file_mime(event)
}

fn has_video_file_mime(event: &Event) -> bool {
    tag_values(event, "m").any(|value| VIDEO_FILE_MIME_TYPES.contains(&value))
}

/// Addressable kinds must name a `d` identifier or the event is skipped
/// (nostr_video_event_mapper.dart `_identifier`); the value is trimmed like
/// NostrEventIdentifier.parse.
fn addressable_identifier(event: &Event) -> Option<Option<String>> {
    if event.kind.as_u16() < 30_000 {
        return Some(None);
    }
    let tag = event
        .tags
        .iter()
        .find(|tag| tag.as_slice().first().map(String::as_str) == Some("d"))?;
    let value = tag.as_slice().get(1)?.trim();
    (!value.is_empty()).then(|| Some(value.to_owned()))
}

fn parsed_post(
    event: &Event,
    identifier: Option<String>,
    media: NativeMediaMetadata,
) -> ParsedVideoPost {
    let urls = media_urls(&media);
    ParsedVideoPost {
        event_id: event.id.to_hex(),
        author_pubkey: event.pubkey.to_hex(),
        kind: event.kind.as_u16(),
        identifier,
        created_at: event.created_at.as_u64(),
        caption: caption_without_urls(&event.content, &urls),
        title: media.title.clone(),
        hashtags: post_hashtags(event),
        dimensions: media.extras.dimensions,
        blurhash: media.extras.blurhash.clone(),
        thumbnail_url: media.extras.image_url.clone(),
        meta: video_meta(media, urls),
    }
}

fn media_urls(media: &NativeMediaMetadata) -> Vec<String> {
    let mut urls = vec![media.url.clone()];
    urls.extend(media.fallback_urls.iter().cloned());
    urls
}

fn video_meta(media: NativeMediaMetadata, urls: Vec<String>) -> VideoMeta {
    VideoMeta {
        urls,
        delivery: delivery_kind(media.delivery),
        sha256: media.expected_digest,
        size_bytes: media.extras.size_bytes,
        duration_ms: media.extras.duration_ms,
    }
}

fn delivery_kind(delivery: NativeVideoDelivery) -> DeliveryKind {
    match delivery {
        NativeVideoDelivery::Hls => DeliveryKind::Hls,
        NativeVideoDelivery::Progressive => DeliveryKind::Progressive,
    }
}

/// t-tags first, then content hashtags, deduplicated in first-seen order
/// (nostr_video_event_mapper.dart `_hashtags`).
fn post_hashtags(event: &Event) -> Vec<String> {
    let mut found = Vec::new();
    for raw in tag_values(event, "t") {
        if let Some(tag) = normalized_hashtag(raw) {
            push_unique(&mut found, tag);
        }
    }
    for tag in content_hashtags(&event.content) {
        push_unique(&mut found, tag);
    }
    found
}
