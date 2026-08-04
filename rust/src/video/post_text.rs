//! Display text derived from a video post's note, mirroring
//! `captionWithoutMediaUrls` in lib/features/video_catalog/data/
//! nostr_video_media.dart and lib/features/video_catalog/domain/
//! video_hashtags.dart.

use crate::video::native_text::bounded_native_text;

/// Links that became the playable media are noise once the video renders:
/// strip them and collapse whitespace, like the Dart caption helper.
pub fn caption_without_urls(content: &str, urls: &[String]) -> String {
    let mut caption = content.to_owned();
    for url in urls {
        caption = caption.replace(url.as_str(), " ");
    }
    let collapsed = caption.split_whitespace().collect::<Vec<_>>().join(" ");
    bounded_native_text(&collapsed)
}

/// Dart `normalizeHashtag`: trim, lowercase, strip one leading '#'.
pub fn normalized_hashtag(raw: &str) -> Option<String> {
    let lowered = raw.trim().to_lowercase();
    let value = lowered.strip_prefix('#').unwrap_or(&lowered);
    (!value.is_empty()).then(|| bounded_native_text(value))
}

/// Dart `extractHashtags` pattern `#([\p{L}\p{N}_]+)`, lowered, in order.
/// Duplicates are kept; callers dedupe while merging with t-tags.
pub fn content_hashtags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for piece in content.split('#').skip(1) {
        let end = piece
            .find(|value: char| !is_hashtag_char(value))
            .unwrap_or(piece.len());
        if end > 0 {
            tags.push(bounded_native_text(&piece[..end].to_lowercase()));
        }
    }
    tags
}

fn is_hashtag_char(value: char) -> bool {
    value.is_alphabetic() || value.is_numeric() || value == '_'
}

pub fn push_unique(found: &mut Vec<String>, tag: String) {
    if !found.contains(&tag) {
        found.push(tag);
    }
}
