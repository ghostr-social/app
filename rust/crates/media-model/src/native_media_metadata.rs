use crate::imeta_extras::{imeta_extras, ImetaExtras};
use crate::native_models::NativeVideoDelivery;
use crate::native_text::bounded_native_text;
use crate::video_link_scan::{is_bounded_http_url, is_video_url, url_delivery};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeMediaMetadata {
    pub declared_mime: Option<String>,
    pub delivery: NativeVideoDelivery,
    pub expected_digest: Option<String>,
    pub extras: ImetaExtras,
    pub fallback_urls: Vec<String>,
    pub original_digest: Option<String>,
    pub title: Option<String>,
    pub url: String,
}

/// Lenient discovery imeta parsing: mime is optional when the URL
/// extension proves a video, and a fallback may supply the playable media.
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
        declared_mime: mime.map(normalized_mime),
        delivery: mime_or_url_delivery(mime, &primary),
        expected_digest: expected_digest(tag)?,
        extras: imeta_extras(tag),
        fallback_urls: urls,
        original_digest: imeta_field(tag, "ox").and_then(parse_sha256),
        title: imeta_field(tag, "title").map(bounded_native_text),
        url: primary,
    })
}

fn normalized_mime(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

/// An ordered, deduplicated set of the primary URL and its fallbacks.
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

/// Publishers often omit the mime; a recognized URL extension is enough
/// to identify playable media.
pub(crate) fn playable_media(mime: Option<&str>, url: &str) -> bool {
    match mime {
        Some(mime) => is_video_mime(mime),
        None => is_video_url(url),
    }
}

/// An explicit mime decides delivery; otherwise the URL extension does.
pub(crate) fn mime_or_url_delivery(mime: Option<&str>, url: &str) -> NativeVideoDelivery {
    match mime {
        Some(mime) => media_delivery(mime),
        None => url_delivery(url),
    }
}

fn expected_digest(tag: &[String]) -> Option<Option<String>> {
    let Some(digest) = imeta_field(tag, "x") else {
        return Some(None);
    };
    parse_sha256(digest).map(Some)
}

/// A valid digest is 64 hexadecimal characters, normalized to lowercase.
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

/// Whitespace and ASCII case do not change the meaning of a mime value.
fn is_video_mime(value: &str) -> bool {
    let value = value.trim();
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
    let value = value.trim();
    value.eq_ignore_ascii_case("application/x-mpegurl")
        || value.eq_ignore_ascii_case("application/vnd.apple.mpegurl")
}
