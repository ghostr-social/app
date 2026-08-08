//! The feed shapes the Rust engine serves: main, profile, hashtag, and
//! search, including viewer-scoped routing and visibility rules.

use nostr_sdk::{PublicKey, Timestamp};

use crate::cache::ViewerScope;
use crate::content::parsing::ParsedVideoPost;
use crate::content::social_graph::SocialGraph;
use crate::query::hashtags::normalize_hashtag;
use crate::query::video_filters::{DiscoveryFlow, DiscoveryRequest};

/// One open feed's identity and query recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedSpec {
    /// The home feed. A signed-in viewer's social graph routes the
    /// query to follows' outbox relays and supplies the mute list;
    /// signed out there is no graph, so configured read relays answer an
    /// unscoped global page.
    MainFeed { viewer: Option<PublicKey> },
    /// Every post carrying one hashtag, as typed (with or without `#`).
    Hashtag(String),
    /// The posts of a named set of creators: one creator for a profile
    /// grid, or every followed creator for the Following feed.
    Profile(Vec<PublicKey>),
    /// A viewer search query, as typed.
    Search(String),
}

impl FeedSpec {
    /// The discovery request answering one page of this feed; `None` when
    /// the spec can never produce content (blank query or empty hashtag).
    /// The viewer's graph only *routes* the main feed: its follows pick
    /// the relays (NIP-65 outbox), never the authors the query filters
    /// by, so a follows-routed page still carries posts by creators the
    /// viewer does not follow.
    pub fn page_request(
        &self,
        older_than: Option<Timestamp>,
        graph: &SocialGraph,
    ) -> Option<DiscoveryRequest> {
        match self {
            Self::MainFeed { viewer } => Some(DiscoveryRequest {
                routing_authors: routing_follows(viewer, graph),
                viewer: viewer_scope(viewer),
                flow: DiscoveryFlow::Continuous,
                ..request(older_than)
            }),
            Self::Profile(creators) => Some(DiscoveryRequest {
                authors: creators.clone(),
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
    /// a profile grid shows exactly its creators, muted or not
    /// (`ProfileDetailsPolicy.build` filters only by creator id).
    pub(crate) fn accepts(&self, post: &ParsedVideoPost, graph: &SocialGraph) -> bool {
        match self {
            Self::MainFeed { viewer } => accepts_main_feed(viewer, post, graph),
            Self::Profile(creators) => written_by(post, creators),
            Self::Hashtag(raw) => accepts_query(post, graph, carries_tag(post, raw)),
            Self::Search(raw) => accepts_query(post, graph, matches_search(post, raw)),
        }
    }

    /// Whether an empty older page ends pagination. Canonical feeds
    /// exhaust (`_nextCursor` null in filtered_video_feed_repository.dart);
    /// query feeds never report themselves finished
    /// (query_video_feed_repository.dart).
    pub fn exhausts_on_empty_page(&self) -> bool {
        !self.is_query()
    }

    /// Search and hashtag feeds keep extending while their native hunt stays
    /// open; canonical feeds may retain only a bounded head window.
    pub(crate) fn is_query(&self) -> bool {
        matches!(self, Self::Hashtag(_) | Self::Search(_))
    }
}

/// A signed-out viewer has no follow set, and a graph belonging to
/// someone else must not leak into this feed's routing.
fn routing_follows(viewer: &Option<PublicKey>, graph: &SocialGraph) -> Vec<PublicKey> {
    match viewer {
        Some(viewer) if graph.belongs_to(viewer) => graph.follow_list(),
        _ => Vec::new(),
    }
}

/// The main feed is the only feed that knows who is looking, so it is
/// the only one that scopes the session's event pool. Signing out is a
/// scope change of its own: a signed-out feed must not answer from the
/// rows the previous viewer's session gathered.
fn viewer_scope(viewer: &Option<PublicKey>) -> ViewerScope {
    match viewer {
        Some(viewer) => ViewerScope::SignedIn(*viewer),
        None => ViewerScope::SignedOut,
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

fn written_by(post: &ParsedVideoPost, creators: &[PublicKey]) -> bool {
    creators
        .iter()
        .any(|creator| post.author_pubkey == creator.to_hex())
}

fn accepts_main_feed(
    viewer: &Option<PublicKey>,
    post: &ParsedVideoPost,
    graph: &SocialGraph,
) -> bool {
    viewer.is_none() || !author_muted(post, graph)
}

fn accepts_query(post: &ParsedVideoPost, graph: &SocialGraph, matches: bool) -> bool {
    !author_muted(post, graph) && matches
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
