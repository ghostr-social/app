//! Parsed Nostr events as domain objects: the posts worth playing, the
//! profiles behind them, and the follow graph that routes queries.
//! This is the layer everything above reads; it depends on nothing but
//! the shared retrieval vocabulary.

pub mod candidates;
pub mod parsing;
pub mod profiles;
mod renditions;
pub mod social_graph;
