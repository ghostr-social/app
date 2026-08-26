//! Generated FFI feed contract: typed feed specs in, ordered post
//! snapshots out.

use crate::api::delivery_types::FfiMediaDelivery;

/// The shape of one feed opened by Dart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiFeedKind {
    Main,
    Hashtag,
    Search,
    Profile,
    Following,
}

/// One feed to open, as Dart names it.
///
/// `Main` reads the signed-in viewer from `viewer_pubkey` (hex or npub);
/// `Hashtag` reads the tag from `value` (leading `#` optional); `Search` reads the
/// query from `value`, as typed; `Profile` reads every creator key from `creators`
/// — one for a profile grid, the whole follow set for the Following feed.
#[derive(Clone, Debug)]
pub struct FfiFeedSpec {
    pub kind: FfiFeedKind,
    pub value: Option<String>,
    pub creators: Vec<String>,
    pub viewer_pubkey: Option<String>,
}

/// The creator identity a feed row renders, including the
/// shortened-npub fallback when no metadata is known.
#[derive(Clone, Debug)]
pub struct FfiFeedCreator {
    pub(crate) pubkey: String,
    pub(crate) display_name: String,
    pub(crate) handle: String,
    pub(crate) avatar_url: Option<String>,
}

/// The outer NIP-18 occurrence that lifted an original into the feed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiFeedRepostTarget {
    SpecificEvent,
    Coordinate,
}

/// The outer NIP-18 occurrence that lifted an original into the feed.
#[derive(Clone, Debug)]
pub struct FfiFeedRepost {
    pub(crate) event_id: String,
    pub(crate) event_kind: u16,
    pub(crate) target: FfiFeedRepostTarget,
    pub(crate) reposted_at: u64,
    pub(crate) reposter: FfiFeedCreator,
}

/// Pixel dimensions from the post's imeta, when declared.
#[derive(Clone, Copy, Debug)]
pub struct FfiMediaDim {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Playable media of one post. `delivery` round-trips with
/// `FfiFocusItem.delivery`.
#[derive(Clone, Debug)]
pub struct FfiFeedMedia {
    /// Playback candidates in preference order (imeta url + fallbacks).
    pub(crate) urls: Vec<String>,
    pub(crate) delivery: FfiMediaDelivery,
    pub(crate) sha256: Option<String>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) dim: Option<FfiMediaDim>,
    pub(crate) blurhash: Option<String>,
    pub(crate) thumb_url: Option<String>,
}

/// One assembled feed row. `post_id` is the gateway-safe id Dart hands
/// back to `ffi_update_focus` / `ffi_playback_url`; it stays stable
/// across addressable revisions of the same video.
#[derive(Clone, Debug)]
pub struct FfiFeedPost {
    pub(crate) post_id: String,
    pub(crate) event_id: String,
    /// The Nostr kind of `event_id`. Named apart from `FfiFeedSpec.kind`
    /// (a feed shape) and `FfiDeliveryEvent.kind` (an event type); with
    /// `identifier` it completes the reference Dart's social writes
    /// address.
    pub(crate) event_kind: u16,
    /// The addressable `d` tag, present exactly for kinds 30000-39999.
    pub(crate) identifier: Option<String>,
    /// The exact published `d` tag used in Nostr coordinates.
    pub(crate) published_identifier: Option<String>,
    /// Unix seconds of the post's newest event.
    pub(crate) created_at: u64,
    pub(crate) feed_sort_at: u64,
    pub(crate) signed_event_json: Option<String>,
    pub(crate) is_protected: bool,
    pub(crate) repost: Option<FfiFeedRepost>,
    pub(crate) caption: String,
    pub(crate) title: Option<String>,
    pub(crate) hashtags: Vec<String>,
    pub(crate) creator: FfiFeedCreator,
    pub(crate) media: FfiFeedMedia,
}

/// How far the snapshot's page got. Row counts cannot answer that — a
/// plan that resolved to nothing and a plan still in flight both show
/// zero posts — so the stage is what a pull-shaped caller waits on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiFeedStage {
    /// A retrieval is in flight; `posts` may still grow.
    Loading,
    /// Every query of the page resolved: `posts` is the whole page.
    Settled,
    /// The page's primary query failed; Dart renders a failure rather
    /// than treating the partial rows as an empty feed.
    Failed,
}

/// One feed-stream update: the feed's full ordered snapshot, newest first.
///
/// Snapshots replace the previous list wholesale — chosen over diffs so the Dart
/// side has nothing to reconcile; `revision` is the feed store's monotonic
/// revision for cheap deduplication.
#[derive(Clone, Debug)]
pub struct FfiFeedUpdate {
    pub(crate) feed_id: String,
    pub(crate) revision: u64,
    pub(crate) stage: FfiFeedStage,
    pub(crate) posts: Vec<FfiFeedPost>,
}
