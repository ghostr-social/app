//! Nostr discovery: relay querying, outbox routing, search, and feed
//! assembly for the media engine. Modules register below as they land.

pub mod candidate_registry;
pub mod event_parsing;

mod cache_fallback;
pub mod control_loop;
pub mod discovery_scheduler;
pub mod event_cache;
mod event_cache_database;
mod event_cache_merge;
mod event_cache_session;
pub mod event_queries;
pub mod feed_assembly;
pub(crate) mod feed_cursor;
pub mod feed_spec;
pub mod feed_store;
mod feed_store_cursor;
pub mod hashtags;
mod live_search_relays;
pub mod outbox_bootstrap;
pub mod outbox_directory;
pub mod outbox_plans;
mod outbox_relay_list;
pub mod pagination;
pub mod plan_executor;
pub mod profile_store;
mod relay_fetch;
mod relay_io;
mod relay_plan_collector;
pub mod relay_plan_executor;
mod relay_plan_routes;
pub(crate) mod relay_pool_owner;
mod relay_pool_roles;
mod relay_pool_route;
mod relay_pool_transition;
mod relay_registration;
mod relay_removal;
mod relay_role_book;
pub mod relay_url;
pub mod retrieval_queue;
mod scheduler_commands;
mod scheduler_feeds;
mod scheduler_hunt;
mod scheduler_loop;
mod scheduler_plans;
mod scheduler_progress;
mod scheduler_queries;
mod scheduler_retry;
mod scheduler_session;
pub mod search_queries;
pub(crate) mod session_generation;
pub mod social_graph;
pub mod trending;
pub mod video_filters;

#[cfg(test)]
mod tests;
