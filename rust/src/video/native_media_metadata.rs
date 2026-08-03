use crate::video::native_models::NativeVideoDelivery;
use crate::video::native_text::{bounded_native_text, MAX_NATIVE_URL_BYTES};
use std::collections::HashSet;

const MAX_NATIVE_FALLBACK_URLS: usize = 4;

pub struct NativeMediaMetadata {
    pub delivery: NativeVideoDelivery,
    pub expected_digest: Option<String>,
    pub fallback_urls: Vec<String>,
    pub title: Option<String>,
    pub url: String,
}

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
        fallback_urls: fallback_urls(tag, url),
        title: imeta_field(tag, "title").map(bounded_native_text),
        url: url.to_owned(),
    })
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
    (digest.len() == 64 && digest.chars().all(|value| value.is_ascii_hexdigit()))
        .then(|| Some(digest.to_ascii_lowercase()))
}

fn imeta_field<'a>(tag: &'a [String], name: &str) -> Option<&'a str> {
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

fn is_http_url(value: &str) -> bool {
    reqwest::Url::parse(value)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some())
        .unwrap_or(false)
}

fn is_bounded_http_url(value: &str) -> bool {
    value.len() <= MAX_NATIVE_URL_BYTES && is_http_url(value)
}
