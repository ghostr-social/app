//! Direct video links in note text, mirroring the Dart scraper in
//! lib/features/video_catalog/data/nostr_video_media.dart (`_fromText`,
//! `_isVideoUrl`, `_urlDelivery`).

use crate::video::native_models::NativeVideoDelivery;
use crate::video::native_text::MAX_NATIVE_URL_BYTES;

/// Product decision mirrored from nostr_video_media.dart
/// `_videoExtensions`: exactly these path suffixes count as video.
const VIDEO_EXTENSIONS: [&str; 5] = [".mp4", ".m4v", ".webm", ".mov", ".m3u8"];

/// Dart `_trailingPunctuation`: characters dropped from a link's tail.
const TRAILING_PUNCTUATION: &[char] = &['.', ',', ';', ':', '!', '?', ')', ']', '}', '\'', '"'];

/// First direct video link in a note, per Dart's `https?://\S+` scan:
/// leftmost token wins, and a rejected token is skipped whole.
pub fn first_video_link(content: &str) -> Option<String> {
    let mut cursor = 0;
    while let Some(offset) = content[cursor..].find("http") {
        let start = cursor + offset;
        if !has_link_scheme(&content[start..]) {
            cursor = start + 1;
            continue;
        }
        let token = link_token(&content[start..]);
        let link = token.trim_end_matches(TRAILING_PUNCTUATION);
        if is_video_url(link) {
            return Some(link.to_owned());
        }
        cursor = start + token.len();
    }
    None
}

fn has_link_scheme(rest: &str) -> bool {
    rest.starts_with("http://") || rest.starts_with("https://")
}

fn link_token(rest: &str) -> &str {
    rest.find(char::is_whitespace)
        .map_or(rest, |end| &rest[..end])
}

pub fn is_video_url(url: &str) -> bool {
    is_bounded_http_url(url) && video_url_extension(url).is_some()
}

pub fn url_delivery(url: &str) -> NativeVideoDelivery {
    if video_url_extension(url) == Some(".m3u8") {
        NativeVideoDelivery::Hls
    } else {
        NativeVideoDelivery::Progressive
    }
}

fn video_url_extension(url: &str) -> Option<&'static str> {
    let path = reqwest::Url::parse(url).ok()?.path().to_ascii_lowercase();
    VIDEO_EXTENSIONS
        .iter()
        .find(|extension| path.ends_with(*extension))
        .copied()
}

pub(crate) fn is_http_url(value: &str) -> bool {
    reqwest::Url::parse(value)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some())
        .unwrap_or(false)
}

/// The URL byte bound is Rust-side hardening on top of the Dart spec.
pub(crate) fn is_bounded_http_url(value: &str) -> bool {
    value.len() <= MAX_NATIVE_URL_BYTES && is_http_url(value)
}
