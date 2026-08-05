//! Pure mapping between FFI feed payloads and discovery types. No IO,
//! no state — fully covered by the unit tests in `crate::api::tests`.

use crate::api::delivery_types::FfiMediaDelivery;
use crate::api::feed_types::{
    FfiFeedCreator, FfiFeedKind, FfiFeedMedia, FfiFeedPost, FfiFeedSpec, FfiMediaDim,
};
use crate::discovery::event_parsing::ParsedVideoPost;
use crate::discovery::feed_assembly::post_coordinate;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::feed_store::FeedId;
use crate::discovery::profile_store::{CreatorProfile, ProfileStore};
use crate::engine::DeliveryKind;
use anyhow::{anyhow, bail, Result};
use nostr_sdk::PublicKey;
use sha2::{Digest, Sha256};

pub(crate) fn parse_feed_spec(spec: &FfiFeedSpec) -> Result<FeedSpec> {
    match spec.kind {
        FfiFeedKind::Main => Ok(FeedSpec::MainFeed {
            viewer: optional_key(spec.viewer_pubkey.as_deref(), "viewer_pubkey")?,
        }),
        FfiFeedKind::Profile => Ok(FeedSpec::Profile(parsed_keys(&spec.creators)?)),
        FfiFeedKind::Hashtag => Ok(FeedSpec::Hashtag(required_value(spec, "hashtag")?)),
        FfiFeedKind::Search => Ok(FeedSpec::Search(required_value(spec, "search")?)),
    }
}

/// Feed handles cross the FFI as the numeric strings `ffi_open_feed`
/// returned.
pub(crate) fn parse_feed_id(raw: &str) -> Result<FeedId> {
    raw.parse::<u64>()
        .map(FeedId)
        .map_err(|_| anyhow!("feed ids are the numeric strings ffi_open_feed returned"))
}

/// Dart has already dropped the ids that decode to nothing
/// (rust_feed_spec_builder.dart), so an empty list means the caller
/// asked for a feed that cannot exist.
fn parsed_keys(raw: &[String]) -> Result<Vec<PublicKey>> {
    if raw.is_empty() {
        bail!("profile feeds need at least one creator");
    }
    raw.iter()
        .map(|creator| public_key(creator, "creators"))
        .collect()
}

/// A key Dart may leave out: a missing `main` viewer is a signed-out
/// session, but a viewer that is present must still parse.
fn optional_key(raw: Option<&str>, field: &str) -> Result<Option<PublicKey>> {
    raw.map(|raw| public_key(raw, field)).transpose()
}

fn public_key(raw: &str, field: &str) -> Result<PublicKey> {
    PublicKey::parse(raw).map_err(|error| anyhow!("{field} is not a public key: {error}"))
}

fn required_value(spec: &FfiFeedSpec, kind: &str) -> Result<String> {
    spec.value
        .clone()
        .ok_or_else(|| anyhow!("{kind} feeds need a value"))
}

/// The gateway-safe post id (`validate_post_id` charset): the sha256
/// hex of the same-video coordinate, so every revision of an
/// addressable video keeps one id and one cache entry.
pub(crate) fn post_gateway_id(post: &ParsedVideoPost) -> String {
    format!("{:x}", Sha256::digest(post_coordinate(post).as_bytes()))
}

/// The stored identity of one post's creator; parsed posts always
/// carry the valid author hex their signed event was keyed by.
pub(crate) fn resolved_creator(profiles: &ProfileStore, post: &ParsedVideoPost) -> CreatorProfile {
    let author =
        PublicKey::from_hex(&post.author_pubkey).expect("parsed posts carry a valid author key");
    profiles.profile(&author)
}

/// One parsed post plus its resolved creator as the full FFI row.
pub(crate) fn feed_post(post: &ParsedVideoPost, creator: CreatorProfile) -> FfiFeedPost {
    FfiFeedPost {
        post_id: post_gateway_id(post),
        event_id: post.event_id.clone(),
        event_kind: post.kind,
        identifier: post.identifier.clone(),
        created_at: post.created_at,
        caption: post.caption.clone(),
        title: post.title.clone(),
        hashtags: post.hashtags.clone(),
        creator: feed_creator(&post.author_pubkey, creator),
        media: feed_media(post),
    }
}

fn feed_creator(pubkey: &str, profile: CreatorProfile) -> FfiFeedCreator {
    FfiFeedCreator {
        pubkey: pubkey.to_owned(),
        display_name: profile.display_name,
        handle: profile.handle,
        avatar_url: profile.avatar_url,
    }
}

fn feed_media(post: &ParsedVideoPost) -> FfiFeedMedia {
    FfiFeedMedia {
        urls: post.meta.urls.clone(),
        delivery: ffi_delivery(post.meta.delivery),
        sha256: post.meta.sha256.clone(),
        size_bytes: post.meta.size_bytes,
        duration_ms: post.meta.duration_ms,
        dim: post
            .dimensions
            .map(|(width, height)| FfiMediaDim { width, height }),
        blurhash: post.blurhash.clone(),
        thumb_url: post.thumbnail_url.clone(),
    }
}

/// Round-trips with the FFI-to-engine mapping in `focus_mapping`.
fn ffi_delivery(kind: DeliveryKind) -> FfiMediaDelivery {
    match kind {
        DeliveryKind::Progressive => FfiMediaDelivery::Progressive,
        DeliveryKind::Hls => FfiMediaDelivery::Hls,
    }
}
