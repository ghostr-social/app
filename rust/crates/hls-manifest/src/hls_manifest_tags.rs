use crate::hls_manifest::HlsResourceKind;
use crate::hls_manifest_attributes::quoted_attribute;
use crate::hls_manifest_names::{attribute, tag};
use anyhow::{bail, Result};

pub(crate) enum HlsTagAction {
    Pass,
    NextUri(HlsResourceKind),
    RewriteUri {
        kind: HlsResourceKind,
        required: bool,
    },
}

/// Tags that carry no resource reference, so the gateway can copy the line
/// through as written. Membership is the claim that the tag is inert; anything
/// not listed here and not handled above is rejected.
const PASSTHROUGH_TAGS: &[&str] = &[
    tag::EXTM3U,
    tag::EXTINF,
    tag::EXT_X_VERSION,
    tag::EXT_X_BYTERANGE,
    tag::EXT_X_DISCONTINUITY,
    tag::EXT_X_PROGRAM_DATE_TIME,
    tag::EXT_X_GAP,
    tag::EXT_X_BITRATE,
    tag::EXT_X_TARGETDURATION,
    tag::EXT_X_MEDIA_SEQUENCE,
    tag::EXT_X_DISCONTINUITY_SEQUENCE,
    tag::EXT_X_ENDLIST,
    tag::EXT_X_PLAYLIST_TYPE,
    tag::EXT_X_I_FRAMES_ONLY,
    tag::EXT_X_INDEPENDENT_SEGMENTS,
    tag::EXT_X_START,
    tag::EXT_X_SERVER_CONTROL,
    tag::EXT_X_PART_INF,
    tag::EXT_X_SKIP,
    tag::EXT_X_IMAGES_ONLY,
    tag::EXT_X_TILES,
    tag::EXT_X_ALLOW_CACHE,
];

/// `#EXT-X-DATERANGE` attributes that point at interstitial content the player
/// would fetch outside the gateway.
const INTERSTITIAL_ATTRIBUTES: &[&str] = &[
    attribute::URI,
    attribute::X_ASSET_URI,
    attribute::X_ASSET_LIST,
];

pub(crate) fn action(line: &str) -> Result<HlsTagAction> {
    let name = line.split_once(':').map_or(line, |(name, _)| name);
    match name {
        tag::EXT_X_CONTENT_STEERING | tag::EXT_X_DEFINE => {
            bail!("{name} is not supported by the secure HLS gateway")
        }
        tag::EXT_X_STREAM_INF => Ok(HlsTagAction::NextUri(HlsResourceKind::Manifest)),
        tag::EXT_X_MEDIA => rewrite(HlsResourceKind::Manifest, false),
        tag::EXT_X_I_FRAME_STREAM_INF | tag::EXT_X_IMAGE_STREAM_INF => {
            rewrite(HlsResourceKind::Manifest, true)
        }
        tag::EXT_X_RENDITION_REPORT => rewrite(HlsResourceKind::Manifest, true),
        tag::EXT_X_KEY | tag::EXT_X_SESSION_KEY | tag::EXT_X_SESSION_DATA => {
            rewrite(HlsResourceKind::Asset, false)
        }
        tag::EXT_X_MAP | tag::EXT_X_PART | tag::EXT_X_PRELOAD_HINT => {
            rewrite(HlsResourceKind::Asset, true)
        }
        tag::EXT_X_DATERANGE => validate_date_range(line),
        _ if is_safe_tag(name) || !name.starts_with(tag::EXT_PREFIX) => Ok(HlsTagAction::Pass),
        _ => bail!("unsupported HLS tag {name}"),
    }
}

fn rewrite(kind: HlsResourceKind, required: bool) -> Result<HlsTagAction> {
    Ok(HlsTagAction::RewriteUri { kind, required })
}

fn validate_date_range(line: &str) -> Result<HlsTagAction> {
    for name in INTERSTITIAL_ATTRIBUTES {
        if quoted_attribute(line, name)?.is_some() {
            bail!("HLS interstitial resources are not supported")
        }
    }
    Ok(HlsTagAction::Pass)
}

fn is_safe_tag(name: &str) -> bool {
    PASSTHROUGH_TAGS.contains(&name)
}
