//! Converts one accepted Nostr event into one playable video post.
//! Feed assembly consumes [`video_post_from_event`]; nothing here does IO.

use ghostr_engine::{DeliveryKind, VideoMeta};
use ghostr_media_model::event_identity::VIDEO_KINDS;
use ghostr_media_model::native_media_metadata::NativeMediaMetadata;
use ghostr_media_model::native_models::NativeVideoDelivery;
use ghostr_media_model::nostr_event_media::{event_media, tag_values};
use ghostr_media_model::post_text::{
    caption_without_urls, content_hashtags, normalized_hashtag, push_unique,
};
use nostr_sdk::Event;

pub const NOTE_KIND: u16 = 1;
pub const FILE_METADATA_KIND: u16 = 1063;

/// Server-side `#m` values accepted by the kind-1063 discovery query.
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
    /// The addressable `d` value trimmed, as Dart addresses social
    /// writes with it (`NostrEventIdentifier.parse`).
    pub identifier: Option<String>,
    /// The same `d` value exactly as published: two events whose
    /// identifiers differ only in padding are two different
    /// coordinates, so the published value remains available for
    /// canonical coordinate comparison.
    pub published_identifier: Option<String>,
    pub created_at: u64,
    pub caption: String,
    pub title: Option<String>,
    pub hashtags: Vec<String>,
    pub dimensions: Option<(u32, u32)>,
    pub blurhash: Option<String>,
    pub thumbnail_url: Option<String>,
    pub meta: VideoMeta,
}

impl ParsedVideoPost {
    /// The same-video identity of this post: addressable video revisions
    /// share `kind:pubkey:identifier`, everything else is its event id.
    /// The identifier is compared exactly as published: padding names a
    /// distinct coordinate.
    pub fn coordinate(&self) -> String {
        if !(30_000..40_000).contains(&u32::from(self.kind)) {
            return self.event_id.clone();
        }
        match &self.published_identifier {
            Some(identifier) => {
                format!("{}:{}:{}", self.kind, self.author_pubkey, identifier)
            }
            None => self.event_id.clone(),
        }
    }
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

/// Addressable kinds must name a non-blank `d` identifier or the event is
/// skipped. The value stays exact for coordinates and is trimmed for display.
fn addressable_identifier(event: &Event) -> Option<Option<String>> {
    if event.kind.as_u16() < 30_000 {
        return Some(None);
    }
    let tag = event
        .tags
        .iter()
        .find(|tag| tag.as_slice().first().map(String::as_str) == Some("d"))?;
    let value = tag.as_slice().get(1)?;
    (!value.trim().is_empty()).then(|| Some(value.clone()))
}

fn parsed_post(
    event: &Event,
    published: Option<String>,
    media: NativeMediaMetadata,
) -> ParsedVideoPost {
    let urls = media_urls(&media);
    ParsedVideoPost {
        event_id: event.id.to_hex(),
        author_pubkey: event.pubkey.to_hex(),
        kind: event.kind.as_u16(),
        identifier: published.as_deref().map(|raw| raw.trim().to_owned()),
        published_identifier: published,
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

/// t-tags first, then content hashtags, deduplicated in first-seen order.
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
