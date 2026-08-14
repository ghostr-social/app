use nostr_sdk::PublicKey;

use crate::content::parsing::ParsedVideoPost;
use crate::content::social_graph::SocialGraph;
use crate::feed::spec::{normalized_leading_hashtag, FeedSpec};
use crate::query::hashtags::normalize_hashtag;

pub(super) fn accepts(spec: &FeedSpec, post: &ParsedVideoPost, graph: &SocialGraph) -> bool {
    match spec {
        FeedSpec::MainFeed { viewer } => main_accepts(post, graph, viewer.is_none()),
        FeedSpec::Profile(creators) => profile_accepts(post, creators),
        FeedSpec::Following { follows, .. } => following_accepts(post, graph, follows),
        FeedSpec::Hashtag(raw) => hashtag_accepts(post, graph, raw),
        FeedSpec::Search(raw) => search_accepts(post, graph, raw),
    }
}

fn main_accepts(post: &ParsedVideoPost, graph: &SocialGraph, signed_out: bool) -> bool {
    post.repost.is_none() && (signed_out || !actor_muted(post, graph))
}

fn profile_accepts(post: &ParsedVideoPost, creators: &[PublicKey]) -> bool {
    post.repost.is_none() && written_by(post, creators)
}

fn following_accepts(post: &ParsedVideoPost, graph: &SocialGraph, follows: &[PublicKey]) -> bool {
    follows_actor(post, follows) && !actor_muted(post, graph)
}

fn hashtag_accepts(post: &ParsedVideoPost, graph: &SocialGraph, raw: &str) -> bool {
    query_accepts(post, graph, carries_tag(post, raw))
}

fn search_accepts(post: &ParsedVideoPost, graph: &SocialGraph, raw: &str) -> bool {
    query_accepts(post, graph, matches_search(post, raw))
}

fn follows_actor(post: &ParsedVideoPost, follows: &[PublicKey]) -> bool {
    follows.iter().any(|follow| {
        let actor = post
            .repost
            .as_ref()
            .map_or(post.author_pubkey.as_str(), |repost| {
                repost.reposter_pubkey.as_str()
            });
        actor == follow.to_hex()
    })
}

fn written_by(post: &ParsedVideoPost, creators: &[PublicKey]) -> bool {
    creators
        .iter()
        .any(|creator| post.author_pubkey == creator.to_hex())
}

fn query_accepts(post: &ParsedVideoPost, graph: &SocialGraph, matches: bool) -> bool {
    post.repost.is_none() && !actor_muted(post, graph) && matches
}

fn actor_muted(post: &ParsedVideoPost, graph: &SocialGraph) -> bool {
    is_muted(&post.author_pubkey, graph)
        || post
            .repost
            .as_ref()
            .is_some_and(|repost| is_muted(&repost.reposter_pubkey, graph))
}

fn is_muted(raw: &str, graph: &SocialGraph) -> bool {
    PublicKey::from_hex(raw)
        .map(|author| graph.is_muted(&author))
        .unwrap_or(false)
}

fn carries_tag(post: &ParsedVideoPost, raw: &str) -> bool {
    normalize_hashtag(raw).is_some_and(|tag| post.hashtags.contains(&tag))
}

fn matches_search(post: &ParsedVideoPost, raw: &str) -> bool {
    match normalized_leading_hashtag(&raw.trim().to_lowercase()) {
        Some(tag) => post.hashtags.contains(&tag),
        None => true,
    }
}
