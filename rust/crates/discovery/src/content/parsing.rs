//! Converts one accepted Nostr event into one playable video post.
//! Feed assembly consumes [`video_post_from_event`]; nothing here does IO.

use super::renditions::{progressive_renditions, video_meta};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::VideoMeta;
use ghostr_media_model::event_identity::VIDEO_KINDS;
use ghostr_media_model::native_media_metadata::NativeMediaMetadata;
use ghostr_media_model::nostr_event_media::{event_media, tag_values};
use ghostr_media_model::post_text::{
    caption_without_urls, content_hashtags, normalized_hashtag, push_unique,
};
use nostr_sdk::{Event, JsonUtil};
use std::sync::Arc;

const NOTE_KIND: u16 = 1;
const FILE_METADATA_KIND: u16 = 1063;
pub(crate) const MAX_REPOSTABLE_EVENT_BYTES: usize = 32 * 1024;

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
    /// Feed occurrence time: the outer timestamp for a repost, otherwise
    /// the original publication time.
    pub feed_sort_at: u64,
    pub repost: Option<super::reposts::RepostProvenance>,
    /// Exact signed wire JSON. Protected originals deliberately omit it so
    /// clients cannot embed them in a repost.
    pub signed_event_json: Option<Arc<str>>,
    pub is_protected: bool,
    pub caption: String,
    pub title: Option<String>,
    pub hashtags: Vec<String>,
    pub dimensions: Option<(u32, u32)>,
    pub blurhash: Option<String>,
    pub thumbnail_url: Option<String>,
    pub meta: VideoMeta,
    pub renditions: Vec<VideoRendition>,
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

    pub fn activity_event_id(&self) -> &str {
        self.repost
            .as_ref()
            .map_or(self.event_id.as_str(), |repost| repost.event_id.as_str())
    }
}

pub fn video_post_from_event(event: &Event) -> Option<ParsedVideoPost> {
    if !accepts_kind(event) {
        return None;
    }
    let identifier = addressable_identifier(event)?;
    let media = event_media(event)?;
    let renditions = progressive_renditions(event);
    Some(parsed_post(event, identifier, media, renditions))
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
    renditions: Vec<VideoRendition>,
) -> ParsedVideoPost {
    let meta = video_meta(&media);
    ParsedVideoPost {
        event_id: event.id.to_hex(),
        author_pubkey: event.pubkey.to_hex(),
        kind: event.kind.as_u16(),
        identifier: published.as_deref().map(|raw| raw.trim().to_owned()),
        published_identifier: published,
        created_at: event.created_at.as_u64(),
        feed_sort_at: event.created_at.as_u64(),
        repost: None,
        signed_event_json: signed_event_source(event),
        is_protected: is_protected(event),
        caption: caption_without_urls(&event.content, &meta.urls),
        title: media.title.clone(),
        hashtags: post_hashtags(event),
        dimensions: media.extras.dimensions,
        blurhash: media.extras.blurhash.clone(),
        thumbnail_url: media.extras.image_url.clone(),
        meta,
        renditions,
    }
}

fn signed_event_source(event: &Event) -> Option<Arc<str>> {
    if is_protected(event) {
        return None;
    }
    let json = event.as_json();
    (json.len() <= MAX_REPOSTABLE_EVENT_BYTES).then(|| Arc::from(json))
}

fn is_protected(event: &Event) -> bool {
    event.tags.iter().any(|tag| tag.as_slice() == ["-"])
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
