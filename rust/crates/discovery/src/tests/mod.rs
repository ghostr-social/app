//! Shared fixtures sit here; each folder holds the tests
//! for the matching folder of the crate.

mod event_cache_support;
mod feed_store_support;
mod outbox_support;
mod profile_enrichment_support;
mod relay_io_relay_fixture;
mod scheduler_support;
mod scheduler_wait;
mod scripted_scheduler_support;
mod support;
mod trending_support;

mod cache;
mod execution;
mod feed;
mod outbox;
mod query;
mod relay;
mod scheduler;
