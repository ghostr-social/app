//! FFI contract v1 (plan §2) plus the phase-2 feed/broadcast surface:
//! the narrow surface Dart calls. Only data shapes and thin glue live
//! here — delivery machinery stays in `crate::video`, pure scheduling
//! in `crate::engine`, relay work in `crate::discovery`.

pub mod broadcast_control;
pub mod delivery_events_stream;
pub mod delivery_types;
pub mod engine_control;
pub mod feed_control;
pub mod feed_types;
pub mod feed_updates_stream;
pub mod focus_control;

pub(crate) mod event_snapshots;
pub(crate) mod feed_decisions;
pub(crate) mod feed_mapping;
pub(crate) mod feed_runtime;
pub(crate) mod feed_state;
pub(crate) mod focus_mapping;
pub(crate) mod runtime_registry;
pub(crate) mod tracked_items;

#[cfg(test)]
mod tests;
