//! Optional imeta hints beyond the playable URL, parsed leniently: a
//! malformed field becomes `None` without ever failing the media parse
//! (units and leniency per lib/core/media/video_media_metadata.dart).

use crate::video::native_media_metadata::imeta_field;
use crate::video::native_text::bounded_native_text;
use crate::video::video_link_scan::is_bounded_http_url;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImetaExtras {
    /// imeta `size`: total bytes, when a positive integer.
    pub size_bytes: Option<u64>,
    /// imeta `duration`: seconds (fractional allowed) converted to ms.
    pub duration_ms: Option<u64>,
    /// imeta `dim`: "WxH" pixels, both positive.
    pub dimensions: Option<(u32, u32)>,
    /// imeta `blurhash`: opaque placeholder string.
    pub blurhash: Option<String>,
    /// imeta `image`: HTTP(S) thumbnail URL.
    pub image_url: Option<String>,
}

pub fn imeta_extras(tag: &[String]) -> ImetaExtras {
    ImetaExtras {
        size_bytes: imeta_field(tag, "size").and_then(size_bytes),
        duration_ms: imeta_field(tag, "duration").and_then(duration_ms),
        dimensions: imeta_field(tag, "dim").and_then(dimensions),
        blurhash: imeta_field(tag, "blurhash").map(bounded_native_text),
        image_url: imeta_field(tag, "image").and_then(image_url),
    }
}

fn size_bytes(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok().filter(|bytes| *bytes > 0)
}

fn duration_ms(raw: &str) -> Option<u64> {
    let seconds: f64 = raw.trim().parse().ok()?;
    (seconds.is_finite() && seconds > 0.0).then(|| (seconds * 1000.0).round() as u64)
}

fn dimensions(raw: &str) -> Option<(u32, u32)> {
    let (width, height) = raw.trim().split_once('x')?;
    Some((positive_pixels(width)?, positive_pixels(height)?))
}

fn positive_pixels(raw: &str) -> Option<u32> {
    raw.parse::<u32>().ok().filter(|value| *value > 0)
}

fn image_url(raw: &str) -> Option<String> {
    is_bounded_http_url(raw).then(|| raw.to_owned())
}
