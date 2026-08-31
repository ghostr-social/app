//! Shared fixtures sit here; each folder holds the tests
//! for the matching folder of the crate.

#[path = "../../tests/discovery_support/mod.rs"]
mod discovery_support;
mod external_a;
mod external_b;
#[path = "../../tests/feed_support/mod.rs"]
mod feed_support;

mod blossom_enrichment_support;
mod candidate_registry_bound_test;
mod deletion_enrichment_support;
mod deletion_failure_support;
mod event_cache_support;
mod failing_wipe_database;
mod feed_store_support;
mod outbox_support;
mod profile_enrichment_support;
mod relay_health_owner_fixture;
mod relay_io_auth_fixture;
mod relay_io_auth_retry_fixture;
mod relay_io_concurrent_fixture;
mod relay_io_delayed_fixture;
mod relay_io_health_fixture;
mod relay_io_relay_fixture;
mod repost_reference_fixture;
mod repost_target_executor_support;
mod repost_target_support;
mod scheduler_support;
mod scheduler_wait;
mod scripted_scheduler_support;
mod selective_deletion_support;
mod support;

mod cache;
mod content;
mod execution;
mod feed;
mod outbox;
mod query;
mod relay;
mod scheduler;
