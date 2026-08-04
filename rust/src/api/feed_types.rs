//! Data shapes of the FFI v2 feed surface (plan §2 phase-2 additions):
//! feed specs in, ordered post snapshots out. Additive to the frozen
//! v1 surface in `delivery_types`.

/// One feed to open, as Dart names it. `kind` selects the shape:
/// `"main"` reads the signed-in viewer from `viewer_pubkey` (hex or
/// npub); `"hashtag"` reads the tag from `value` (leading `#`
/// optional); `"profile"` reads the creator key from `value`;
/// `"search"` reads the query from `value`, as typed.
#[derive(Clone, Debug)]
pub struct FfiFeedSpec {
    pub kind: String,
    pub value: Option<String>,
    pub viewer_pubkey: Option<String>,
}

/// The creator identity a feed row renders, including the
/// shortened-npub fallback when no metadata is known.
#[derive(Clone, Debug)]
pub struct FfiFeedCreator {
    pub pubkey: String,
    pub display_name: String,
    pub handle: String,
    pub avatar_url: Option<String>,
}

/// Pixel dimensions from the post's imeta, when declared.
#[derive(Clone, Copy, Debug)]
pub struct FfiMediaDim {
    pub width: u32,
    pub height: u32,
}

/// Playable media of one post. `delivery` round-trips with
/// `FfiFocusItem.delivery` (`"progressive"` / `"hls"`).
#[derive(Clone, Debug)]
pub struct FfiFeedMedia {
    /// Playback candidates in preference order (imeta url + fallbacks).
    pub urls: Vec<String>,
    pub delivery: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
    pub dim: Option<FfiMediaDim>,
    pub blurhash: Option<String>,
    pub thumb_url: Option<String>,
}

/// One assembled feed row. `post_id` is the gateway-safe id Dart hands
/// back to `ffi_update_focus` / `ffi_playback_url`; it stays stable
/// across addressable revisions of the same video.
#[derive(Clone, Debug)]
pub struct FfiFeedPost {
    pub post_id: String,
    pub event_id: String,
    /// The Nostr kind of `event_id`. Named apart from `FfiFeedSpec.kind`
    /// (a feed shape) and `FfiDeliveryEvent.kind` (an event type); with
    /// `identifier` it completes the reference Dart's social writes
    /// address.
    pub event_kind: u16,
    /// The addressable `d` tag, present exactly for kinds 30000-39999.
    pub identifier: Option<String>,
    /// Unix seconds of the post's newest event.
    pub created_at: u64,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub creator: FfiFeedCreator,
    pub media: FfiFeedMedia,
}

/// One feed-stream update: the feed's full ordered snapshot, newest
/// first. Snapshots replace the previous list wholesale — chosen over
/// diffs so the Dart side has nothing to reconcile; `revision` is the
/// feed store's monotonic revision for cheap deduplication.
#[derive(Clone, Debug)]
pub struct FfiFeedUpdate {
    pub feed_id: String,
    pub revision: u64,
    pub posts: Vec<FfiFeedPost>,
}
