//! The engine the FFI surface holds onto between calls.
//!
//! `registry` is the process-wide slot the started engine lives in;
//! `discovery` is the `DiscoveryRuntime` inside it. That type is one
//! type split across files — `start`, `event_queries`, and
//! `accepted_events` each add an `impl` block to it, so an import graph
//! understates how tightly they belong together.

pub(crate) mod accepted_events;
pub(crate) mod configuration;
pub(crate) mod discovery;
pub(super) mod event_queries;
pub(crate) mod registry;
mod start;
pub(crate) mod tracked_items;
