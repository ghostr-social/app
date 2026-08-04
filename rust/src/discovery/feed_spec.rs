//! The feed shapes production actually builds (plan §5.3), each mirroring
//! one Dart repository: the main feed
//! (lib/features/video_catalog/domain/filtered_video_feed_repository.dart,
//! FeedKind.forYou — unscoped query, outbox-routed to the viewer's
//! follows), profile grids (aggregating_video_profile_repository.dart),
//! and hashtag/search query feeds
//! (discovery_video_search_repository.dart via
//! query_video_feed_repository.dart).

use nostr_sdk::{PublicKey, Timestamp};

use crate::discovery::event_parsing::ParsedVideoPost;
use crate::discovery::hashtags::normalize_hashtag;
use crate::discovery::social_graph::SocialGraph;
use crate::discovery::video_filters::DiscoveryRequest;

/// One open feed's identity and query recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedSpec {
    /// The home feed. A signed-in viewer's social graph routes the
    /// query to follows' outbox relays and supplies the mute list;
    /// signed out there is no graph, so the feed degrades to the
    /// unscoped global page the bootstrap relays answer — ndk parity
    /// (lib/platform/nostr/ndk_nostr_outbox_directory.dart knows no
    /// follows without an account and falls back to bootstrap).
    MainFeed { viewer: Option<PublicKey> },
    /// Every post carrying one hashtag, as typed (with or without `#`).
    Hashtag(String),
    /// One creator's grid.
    Profile(PublicKey),
    /// A viewer search query, as typed.
    Search(String),
}

impl FeedSpec {
    /// The discovery request answering one page of this feed; `None` when
    /// the spec can never produce content (blank query, empty hashtag) —
    /// Dart returns an empty page without querying
    /// (`DiscoveryVideoSearchRepository.searchVideos` on a null
    /// normalization).
    pub fn page_request(&self, older_than: Option<Timestamp>) -> Option<DiscoveryRequest> {
        match self {
            Self::MainFeed { .. } => Some(request(older_than)),
            Self::Profile(creator) => Some(DiscoveryRequest {
                authors: vec![*creator],
                ..request(older_than)
            }),
            Self::Hashtag(raw) => hashtag_request(raw, older_than),
            Self::Search(raw) => search_request(raw, older_than),
        }
    }

    /// Whether one assembled post is visible in this feed. Mutes hide
    /// creators from the main and query feeds (video_feed_policy.dart,
    /// `_selectPosts` in discovery_video_search_repository.dart); a
    /// signed-out main feed has no viewer whose mutes could apply, and
    /// a profile grid shows exactly its creator, muted or not
    /// (`ProfileDetailsPolicy.build` filters only by creator id).
    pub fn accepts(&self, post: &ParsedVideoPost, graph: &SocialGraph) -> bool {
        match self {
            Self::MainFeed { viewer } => viewer.is_none() || !author_muted(post, graph),
            Self::Profile(creator) => post.author_pubkey == creator.to_hex(),
            Self::Hashtag(raw) => !author_muted(post, graph) && carries_tag(post, raw),
            Self::Search(raw) => !author_muted(post, graph) && matches_search(post, raw),
        }
    }

    /// Whether an empty older page ends pagination. Canonical feeds
    /// exhaust (`_nextCursor` null in filtered_video_feed_repository.dart);
    /// query feeds never report themselves finished
    /// (query_video_feed_repository.dart).
    pub fn exhausts_on_empty_page(&self) -> bool {
        matches!(self, Self::MainFeed { .. } | Self::Profile(_))
    }
}

fn request(older_than: Option<Timestamp>) -> DiscoveryRequest {
    DiscoveryRequest {
        older_than,
        ..DiscoveryRequest::default()
    }
}

fn hashtag_request(raw: &str, older_than: Option<Timestamp>) -> Option<DiscoveryRequest> {
    let tag = normalize_hashtag(raw)?;
    Some(DiscoveryRequest {
        hashtags: vec![tag],
        ..request(older_than)
    })
}

/// `VideoSearchPolicy.normalize` (trim + lowercase, blank is no query)
/// then the `#`-branch split of `searchVideos`.
fn search_request(raw: &str, older_than: Option<Timestamp>) -> Option<DiscoveryRequest> {
    let normalized = raw.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }
    match leading_hashtag(&normalized) {
        Some(tag) => hashtag_request(&tag, older_than),
        None => Some(DiscoveryRequest {
            search_query: Some(normalized),
            ..request(older_than)
        }),
    }
}

/// `VideoSearchPolicy.hashtag`: only a query starting with `#` that still
/// normalizes to something is a hashtag query — a lone `#` stays text.
fn leading_hashtag(query: &str) -> Option<String> {
    if !query.starts_with('#') {
        return None;
    }
    normalize_hashtag(query)
}

fn author_muted(post: &ParsedVideoPost, graph: &SocialGraph) -> bool {
    PublicKey::from_hex(&post.author_pubkey)
        .map(|author| graph.is_muted(&author))
        .unwrap_or(false)
}

/// Relay tag matching is rechecked locally; NIP-50 text matching is the
/// relay's judgement (`_selectPosts` in
/// discovery_video_search_repository.dart).
fn carries_tag(post: &ParsedVideoPost, raw: &str) -> bool {
    normalize_hashtag(raw).is_some_and(|tag| post.hashtags.contains(&tag))
}

fn matches_search(post: &ParsedVideoPost, raw: &str) -> bool {
    match leading_hashtag(&raw.trim().to_lowercase()) {
        Some(tag) => post.hashtags.contains(&tag),
        None => true,
    }
}
