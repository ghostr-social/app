//! Pure mapping between FFI feed payloads and discovery types. No IO,
//! no state — fully covered by the unit tests in `crate::api::tests`.

use crate::api::feed_types::{FfiFeedCreator, FfiFeedMedia, FfiFeedPost, FfiFeedSpec, FfiMediaDim};
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
    match spec.kind.as_str() {
        "main" => Ok(FeedSpec::MainFeed {
            viewer: parsed_key(spec.viewer_pubkey.as_deref(), "viewer_pubkey")?,
        }),
        "profile" => Ok(FeedSpec::Profile(parsed_key(spec.value.as_deref(), "value")?)),
        "hashtag" => Ok(FeedSpec::Hashtag(required_value(spec)?)),
        "search" => Ok(FeedSpec::Search(required_value(spec)?)),
        other => bail!("unknown feed kind: {other}"),
    }
}

/// Feed handles cross the FFI as the numeric strings `ffi_open_feed`
/// returned.
pub(crate) fn parse_feed_id(raw: &str) -> Result<FeedId> {
    raw.parse::<u64>()
        .map(FeedId)
        .map_err(|_| anyhow!("feed ids are the numeric strings ffi_open_feed returned"))
}

fn parsed_key(raw: Option<&str>, field: &str) -> Result<PublicKey> {
    let raw = raw.ok_or_else(|| anyhow!("this feed kind needs {field}"))?;
    PublicKey::parse(raw).map_err(|error| anyhow!("{field} is not a public key: {error}"))
}

fn required_value(spec: &FfiFeedSpec) -> Result<String> {
    spec.value
        .clone()
        .ok_or_else(|| anyhow!("{} feeds need a value", spec.kind))
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
    let author = PublicKey::from_hex(&post.author_pubkey)
        .expect("parsed posts carry a valid author key");
    profiles.profile(&author)
}

/// One parsed post plus its resolved creator as the full FFI row.
pub(crate) fn feed_post(post: &ParsedVideoPost, creator: CreatorProfile) -> FfiFeedPost {
    FfiFeedPost {
        post_id: post_gateway_id(post),
        event_id: post.event_id.clone(),
        created_at: post.created_at,
        caption: post.caption.clone(),
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
        delivery: delivery_name(post.meta.delivery).to_owned(),
        sha256: post.meta.sha256.clone(),
        size_bytes: post.meta.size_bytes,
        duration_ms: post.meta.duration_ms,
        dim: post.dimensions.map(|(width, height)| FfiMediaDim { width, height }),
        blurhash: post.blurhash.clone(),
        thumb_url: post.thumbnail_url.clone(),
    }
}

/// Round-trips with `parse_delivery_kind` in `focus_mapping`.
fn delivery_name(kind: DeliveryKind) -> &'static str {
    match kind {
        DeliveryKind::Progressive => "progressive",
        DeliveryKind::Hls => "hls",
    }
}
