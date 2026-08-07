//! `ghostr-delivery` — extracted from `rust_lib_ghostr::video`.
//!
//! Deciding what media bytes to fetch, and fetching them. `manager` is
//! the event loop that decides; `chunk` moves the bytes; `probe` learns
//! a source's shape first. `delivery_events` is the control surface
//! callers drive all of it through.

pub mod delivery_events;
pub mod manager;

pub mod chunk;
pub mod probe;

pub mod cache_registry;
mod candidate_priority;
pub mod mutable_priority_queue;
pub mod playback_demand;
pub mod progressive_posts;

pub mod debug;

#[cfg(test)]
mod tests;
