use crate::video::hls_manifest::HlsResourceKind;
use crate::video::hls_manifest_attributes::quoted_attribute;
use anyhow::{bail, Result};

pub(crate) enum HlsTagAction {
    Pass,
    NextUri(HlsResourceKind),
    RewriteUri {
        kind: HlsResourceKind,
        required: bool,
    },
}

pub(crate) fn action(line: &str) -> Result<HlsTagAction> {
    let name = line.split_once(':').map_or(line, |(name, _)| name);
    match name {
        "#EXT-X-CONTENT-STEERING" | "#EXT-X-DEFINE" => {
            bail!("{name} is not supported by the secure HLS gateway")
        }
        "#EXT-X-STREAM-INF" => Ok(HlsTagAction::NextUri(HlsResourceKind::Manifest)),
        "#EXT-X-MEDIA" => rewrite(HlsResourceKind::Manifest, false),
        "#EXT-X-I-FRAME-STREAM-INF" | "#EXT-X-IMAGE-STREAM-INF" => {
            rewrite(HlsResourceKind::Manifest, true)
        }
        "#EXT-X-RENDITION-REPORT" => rewrite(HlsResourceKind::Manifest, true),
        "#EXT-X-KEY" | "#EXT-X-SESSION-KEY" | "#EXT-X-SESSION-DATA" => {
            rewrite(HlsResourceKind::Asset, false)
        }
        "#EXT-X-MAP" | "#EXT-X-PART" | "#EXT-X-PRELOAD-HINT" => {
            rewrite(HlsResourceKind::Asset, true)
        }
        "#EXT-X-DATERANGE" => validate_date_range(line),
        _ if is_safe_tag(name) || !name.starts_with("#EXT") => Ok(HlsTagAction::Pass),
        _ => bail!("unsupported HLS tag {name}"),
    }
}

fn rewrite(kind: HlsResourceKind, required: bool) -> Result<HlsTagAction> {
    Ok(HlsTagAction::RewriteUri { kind, required })
}

fn validate_date_range(line: &str) -> Result<HlsTagAction> {
    for name in ["URI", "X-ASSET-URI", "X-ASSET-LIST"] {
        if quoted_attribute(line, name)?.is_some() {
            bail!("HLS interstitial resources are not supported")
        }
    }
    Ok(HlsTagAction::Pass)
}

fn is_safe_tag(name: &str) -> bool {
    matches!(
        name,
        "#EXTM3U"
            | "#EXTINF"
            | "#EXT-X-VERSION"
            | "#EXT-X-BYTERANGE"
            | "#EXT-X-DISCONTINUITY"
            | "#EXT-X-PROGRAM-DATE-TIME"
            | "#EXT-X-GAP"
            | "#EXT-X-BITRATE"
            | "#EXT-X-TARGETDURATION"
            | "#EXT-X-MEDIA-SEQUENCE"
            | "#EXT-X-DISCONTINUITY-SEQUENCE"
            | "#EXT-X-ENDLIST"
            | "#EXT-X-PLAYLIST-TYPE"
            | "#EXT-X-I-FRAMES-ONLY"
            | "#EXT-X-INDEPENDENT-SEGMENTS"
            | "#EXT-X-START"
            | "#EXT-X-SERVER-CONTROL"
            | "#EXT-X-PART-INF"
            | "#EXT-X-SKIP"
            | "#EXT-X-IMAGES-ONLY"
            | "#EXT-X-TILES"
            | "#EXT-X-ALLOW-CACHE"
    )
}
