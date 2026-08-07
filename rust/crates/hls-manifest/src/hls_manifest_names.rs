//! The HLS vocabulary the gateway recognises, spelled exactly once.
//!
//! Every spec token the manifest rewriter compares a line against is defined
//! here rather than inline at the comparison site, so a tag's spelling has one
//! home and every use of it is symbolic. The rewriter fails closed, which makes
//! a typo here a silent downgrade — an unrecognised tag is rejected, so the
//! manifest breaks loudly instead of a URI escaping the gateway unrewritten.
//!
//! Grouped by the section of the HLS specification that defines each tag, not
//! by how the gateway treats it: the policy lives in `hls_manifest_tags`, and
//! keeping the two orderings independent stops them from drifting into a
//! half-duplicated copy of one another.

/// Tag names as they open a manifest line, leading `#` included.
pub(crate) mod tag {
    /// The prefix every specified tag shares. A `#` line without it is a plain
    /// comment, which the gateway passes through untouched.
    pub(crate) const EXT_PREFIX: &str = "#EXT";

    // Basic tags.
    pub(crate) const EXTM3U: &str = "#EXTM3U";
    pub(crate) const EXT_X_VERSION: &str = "#EXT-X-VERSION";

    // Media segment tags.
    pub(crate) const EXTINF: &str = "#EXTINF";
    pub(crate) const EXT_X_BYTERANGE: &str = "#EXT-X-BYTERANGE";
    pub(crate) const EXT_X_DISCONTINUITY: &str = "#EXT-X-DISCONTINUITY";
    pub(crate) const EXT_X_KEY: &str = "#EXT-X-KEY";
    pub(crate) const EXT_X_MAP: &str = "#EXT-X-MAP";
    pub(crate) const EXT_X_PROGRAM_DATE_TIME: &str = "#EXT-X-PROGRAM-DATE-TIME";
    pub(crate) const EXT_X_DATERANGE: &str = "#EXT-X-DATERANGE";
    pub(crate) const EXT_X_GAP: &str = "#EXT-X-GAP";
    pub(crate) const EXT_X_BITRATE: &str = "#EXT-X-BITRATE";
    pub(crate) const EXT_X_PART: &str = "#EXT-X-PART";

    // Media playlist tags.
    pub(crate) const EXT_X_TARGETDURATION: &str = "#EXT-X-TARGETDURATION";
    pub(crate) const EXT_X_MEDIA_SEQUENCE: &str = "#EXT-X-MEDIA-SEQUENCE";
    pub(crate) const EXT_X_DISCONTINUITY_SEQUENCE: &str = "#EXT-X-DISCONTINUITY-SEQUENCE";
    pub(crate) const EXT_X_ENDLIST: &str = "#EXT-X-ENDLIST";
    pub(crate) const EXT_X_PLAYLIST_TYPE: &str = "#EXT-X-PLAYLIST-TYPE";
    pub(crate) const EXT_X_I_FRAMES_ONLY: &str = "#EXT-X-I-FRAMES-ONLY";
    pub(crate) const EXT_X_IMAGES_ONLY: &str = "#EXT-X-IMAGES-ONLY";
    pub(crate) const EXT_X_SERVER_CONTROL: &str = "#EXT-X-SERVER-CONTROL";
    pub(crate) const EXT_X_PART_INF: &str = "#EXT-X-PART-INF";

    // Multivariant (master) playlist tags.
    pub(crate) const EXT_X_MEDIA: &str = "#EXT-X-MEDIA";
    pub(crate) const EXT_X_STREAM_INF: &str = "#EXT-X-STREAM-INF";
    pub(crate) const EXT_X_I_FRAME_STREAM_INF: &str = "#EXT-X-I-FRAME-STREAM-INF";
    pub(crate) const EXT_X_IMAGE_STREAM_INF: &str = "#EXT-X-IMAGE-STREAM-INF";
    pub(crate) const EXT_X_SESSION_DATA: &str = "#EXT-X-SESSION-DATA";
    pub(crate) const EXT_X_SESSION_KEY: &str = "#EXT-X-SESSION-KEY";
    pub(crate) const EXT_X_CONTENT_STEERING: &str = "#EXT-X-CONTENT-STEERING";

    // Tags valid in either playlist kind.
    pub(crate) const EXT_X_INDEPENDENT_SEGMENTS: &str = "#EXT-X-INDEPENDENT-SEGMENTS";
    pub(crate) const EXT_X_START: &str = "#EXT-X-START";
    pub(crate) const EXT_X_DEFINE: &str = "#EXT-X-DEFINE";

    // Low-latency HLS tags.
    pub(crate) const EXT_X_SKIP: &str = "#EXT-X-SKIP";
    pub(crate) const EXT_X_PRELOAD_HINT: &str = "#EXT-X-PRELOAD-HINT";
    pub(crate) const EXT_X_RENDITION_REPORT: &str = "#EXT-X-RENDITION-REPORT";

    // Tags outside the current specification that players still emit or accept.
    pub(crate) const EXT_X_TILES: &str = "#EXT-X-TILES";
    pub(crate) const EXT_X_ALLOW_CACHE: &str = "#EXT-X-ALLOW-CACHE";
}

/// Attribute names read out of a tag's attribute list.
pub(crate) mod attribute {
    /// The resource a tag points at, and the only attribute the gateway
    /// rewrites to a URI of its own.
    pub(crate) const URI: &str = "URI";

    // Interstitial asset references carried by `#EXT-X-DATERANGE`.
    pub(crate) const X_ASSET_URI: &str = "X-ASSET-URI";
    pub(crate) const X_ASSET_LIST: &str = "X-ASSET-LIST";
}
