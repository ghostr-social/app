//! Shared fixtures sit here; each folder holds the tests
//! for the matching folder of the crate.

mod feed_fixtures;
mod feed_watch_support;
mod hls_runtime_origin;
mod hls_runtime_support;
mod outbox_runtime_support;
mod runtime_fixture;
mod signed_event_fixture;
mod support;

mod broadcast;
#[cfg(feature = "video-debug-web")]
mod debug;
mod delivery;
mod event;
mod feed;
mod runtime;
