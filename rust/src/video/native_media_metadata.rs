use crate::video::imeta_extras::{imeta_extras, ImetaExtras};
use crate::video::native_models::NativeVideoDelivery;
use crate::video::native_text::bounded_native_text;
use crate::video::video_link_scan::{is_bounded_http_url, is_video_url, url_delivery};
use std::collections::HashSet;

const MAX_NATIVE_FALLBACK_URLS: usize = 4;

pub struct NativeMediaMetadata {
    pub delivery: NativeVideoDelivery,
    pub expected_digest: Option<String>,
    pub extras: ImetaExtras,
    pub fallback_urls: Vec<String>,
    pub title: Option<String>,
    pub url: String,
}

/// Strict imeta parse used by the event indexer: the mime is mandatory and
/// the primary URL must stand on its own.
pub fn native_media(tag: &[String]) -> Option<NativeMediaMetadata> {
    if tag.first().map(String::as_str) != Some("imeta") {
        return None;
    }
    let mime = imeta_field(tag, "m")?;
    if mime.len() > 255 || !is_video_mime(mime) {
        return None;
    }
    let url = imeta_field(tag, "url")?;
    if !is_bounded_http_url(url) {
        return None;
    }
    Some(NativeMediaMetadata {
        delivery: media_delivery(mime),
        expected_digest: expected_digest(tag)?,
        extras: imeta_extras(tag),
        fallback_urls: fallback_urls(tag, url),
        title: imeta_field(tag, "title").map(bounded_native_text),
        url: url.to_owned(),
    })
}

/// Dart-parity imeta parse for discovery, mirroring `_tryImeta` in
/// lib/features/video_catalog/data/nostr_video_media.dart: the mime is
/// optional when the URL extension proves a video, and fallback URLs can
/// carry the media when the primary is unusable.
pub fn lenient_native_media(tag: &[String]) -> Option<NativeMediaMetadata> {
    if tag.first().map(String::as_str) != Some("imeta") {
        return None;
    }
    let mut urls = lenient_imeta_urls(tag);
    if urls.is_empty() {
        return None;
    }
    let primary = urls.remove(0);
    let mime = imeta_field(tag, "m");
    if !playable_media(mime, &primary) {
        return None;
    }
    Some(NativeMediaMetadata {
        delivery: mime_or_url_delivery(mime, &primary),
        expected_digest: expected_digest(tag)?,
        extras: imeta_extras(tag),
        fallback_urls: urls,
        title: imeta_field(tag, "title").map(bounded_native_text),
        url: primary,
    })
}

/// Dart `_imetaUrls`: an ordered, deduplicated set of primary + fallbacks.
fn lenient_imeta_urls(tag: &[String]) -> Vec<String> {
    let mut urls: Vec<String> = imeta_field(tag, "url")
        .filter(|url| is_bounded_http_url(url))
        .map(str::to_owned)
        .into_iter()
        .collect();
    let fallbacks = tag
        .iter()
        .skip(1)
        .filter_map(|value| field_value(value, "fallback"));
    for fallback in fallbacks {
        if is_bounded_http_url(fallback) && !urls.iter().any(|url| url == fallback) {
            urls.push(fallback.to_owned());
        }
    }
    urls
}

/// Dart `_playable`: publishers often omit the mime; the URL extension is
/// proof enough.
pub(crate) fn playable_media(mime: Option<&str>, url: &str) -> bool {
    match mime {
        Some(mime) => is_video_mime(mime),
        None => is_video_url(url),
    }
}

/// Dart `_imetaDelivery`: an explicit mime decides, else the extension.
pub(crate) fn mime_or_url_delivery(mime: Option<&str>, url: &str) -> NativeVideoDelivery {
    match mime {
        Some(mime) => media_delivery(mime),
        None => url_delivery(url),
    }
}

fn fallback_urls(tag: &[String], primary: &str) -> Vec<String> {
    let mut seen = HashSet::from([primary.to_owned()]);
    tag.iter()
        .skip(1)
        .filter_map(|value| field_value(value, "fallback"))
        .filter(|url| is_bounded_http_url(url))
        .filter(|url| seen.insert((*url).to_owned()))
        .take(MAX_NATIVE_FALLBACK_URLS)
        .map(str::to_owned)
        .collect()
}

fn expected_digest(tag: &[String]) -> Option<Option<String>> {
    let Some(digest) = imeta_field(tag, "x") else {
        return Some(None);
    };
    parse_sha256(digest).map(Some)
}

/// Digest validity per lib/core/media/video_sha256.dart: 64 hex characters,
/// normalized to lowercase.
pub(crate) fn parse_sha256(raw: &str) -> Option<String> {
    (raw.len() == 64 && raw.chars().all(|value| value.is_ascii_hexdigit()))
        .then(|| raw.to_ascii_lowercase())
}

pub(crate) fn imeta_field<'a>(tag: &'a [String], name: &str) -> Option<&'a str> {
    tag.iter()
        .skip(1)
        .find_map(|value| field_value(value, name))
}

fn field_value<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let (key, field) = value.split_once(char::is_whitespace)?;
    let field = field.trim();
    (key == name && !field.is_empty()).then_some(field)
}

fn is_video_mime(value: &str) -> bool {
    value.to_ascii_lowercase().starts_with("video/") || is_hls_mime(value)
}

fn media_delivery(value: &str) -> NativeVideoDelivery {
    if is_hls_mime(value) {
        NativeVideoDelivery::Hls
    } else {
        NativeVideoDelivery::Progressive
    }
}

fn is_hls_mime(value: &str) -> bool {
    value.eq_ignore_ascii_case("application/x-mpegurl")
        || value.eq_ignore_ascii_case("application/vnd.apple.mpegurl")
}
