//! Nostr discovery: relay querying, outbox routing, search, and feed
//! assembly for the media engine. Modules register below as they land.

pub mod event_parsing;

pub mod control_loop;
pub mod discovery_scheduler;
pub mod feed_assembly;
pub mod feed_spec;
pub mod feed_store;
pub mod hashtags;
pub mod profile_store;
pub mod outbox_directory;
pub mod pagination;
pub mod plan_executor;
pub mod relay_plan_executor;
pub mod relay_url;
pub mod retrieval_queue;
mod scheduler_feeds;
mod scheduler_loop;
pub mod search_queries;
pub mod social_graph;
pub mod trending;
pub mod video_filters;

#[cfg(test)]
mod tests;
