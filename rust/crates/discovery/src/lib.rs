//! Nostr discovery: relay querying, outbox routing, search, and feed
//! assembly for the media engine.
//!
//! The folders below stack. `content` and `query` are the base — parsed
//! events and the filters that ask for them, depending on nothing but
//! the vocabulary in `retrieval_types`. `relay` owns pool membership,
//! `cache` owns what we already have, and `outbox` owns NIP-65 routing.
//! `execution` runs a plan across those three, `feed` holds what comes
//! back, and `scheduler` decides what to run and when.

pub mod plan_executor;
pub mod retrieval_types;
pub mod session_generation;

pub mod cache;
pub mod content;
pub mod execution;
pub mod feed;
pub mod outbox;
pub mod query;
pub mod relay;
pub mod scheduler;

#[cfg(any(test, feature = "test"))]
pub mod test_support;

#[cfg(test)]
mod tests;
