//! Parsed Nostr events as domain objects: the posts worth playing, the
//! profiles behind them, and the follow graph that routes queries.
//! This is the layer everything above reads; it depends on nothing but
//! the shared retrieval vocabulary.

pub mod candidates;
mod deletion_index;
pub mod deletions;
pub mod parsing;
mod pending_deletions;
pub mod profiles;
mod renditions;
mod repost_hint;
pub(crate) mod repost_reference;
pub mod repost_resolution;
pub mod reposts;
pub mod social_graph;
