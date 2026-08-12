//! Playable media resolved from a whole Nostr event: imeta tags first,
//! then NIP-94 top-level file tags, then direct video links in note text.

use crate::event_identity::VIDEO_KINDS;
use crate::imeta_extras::ImetaExtras;
use crate::native_media_metadata::{
    lenient_native_media, mime_or_url_delivery, parse_sha256, playable_media, NativeMediaMetadata,
};
use crate::native_models::NativeVideoDelivery;
use crate::video_link_scan::{first_video_link, is_bounded_http_url, url_delivery};
use nostr_sdk::Event;

pub fn event_media(event: &Event) -> Option<NativeMediaMetadata> {
    imeta_media(event)
        .or_else(|| file_tag_media(event))
        .or_else(|| text_media(event))
}

/// Every independently valid video `imeta`, preserving publisher order.
pub fn event_imeta_media(event: &Event) -> Vec<NativeMediaMetadata> {
    event
        .tags
        .iter()
        .filter_map(|tag| lenient_native_media(tag.as_slice()))
        .collect()
}

/// The first imeta tag that parses wins.
fn imeta_media(event: &Event) -> Option<NativeMediaMetadata> {
    let mut media = event_imeta_media(event);
    if media.is_empty() {
        return None;
    }
    let preferred = VIDEO_KINDS
        .contains(&event.kind.as_u16())
        .then(|| {
            media
                .iter()
                .position(|item| item.delivery == NativeVideoDelivery::Progressive)
        })
        .flatten()
        .unwrap_or(0);
    Some(media.remove(preferred))
}

/// NIP-94 file events carry URL, mime, and digest as top-level tags.
fn file_tag_media(event: &Event) -> Option<NativeMediaMetadata> {
    let url = tag_values(event, "url").next()?;
    if !is_bounded_http_url(url) {
        return None;
    }
    let mime = tag_values(event, "m").next();
    if !playable_media(mime, url) {
        return None;
    }
    Some(NativeMediaMetadata {
        delivery: mime_or_url_delivery(mime, url),
        expected_digest: file_digest(event)?,
        extras: ImetaExtras::default(),
        fallback_urls: Vec::new(),
        title: None,
        url: url.to_owned(),
    })
}

/// A present-but-invalid `x` digest rejects the file tags entirely.
fn file_digest(event: &Event) -> Option<Option<String>> {
    let Some(raw) = tag_values(event, "x").next() else {
        return Some(None);
    };
    parse_sha256(raw.trim()).map(Some)
}

/// The first direct video link becomes the media.
fn text_media(event: &Event) -> Option<NativeMediaMetadata> {
    let url = first_video_link(&event.content)?;
    Some(NativeMediaMetadata {
        delivery: url_delivery(&url),
        expected_digest: None,
        extras: ImetaExtras::default(),
        fallback_urls: Vec::new(),
        title: None,
        url,
    })
}

/// Every value of the named tags; tags shorter than two entries are skipped.
pub fn tag_values<'a>(event: &'a Event, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    event.tags.iter().filter_map(move |tag| {
        let values = tag.as_slice();
        (values.first().map(String::as_str) == Some(name))
            .then(|| values.get(1).map(String::as_str))
            .flatten()
    })
}
