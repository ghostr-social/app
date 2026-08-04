//! FFI contract v1 (plan §2): the narrow, frozen surface Dart calls.
//! Only data shapes and thin glue live here — delivery machinery
//! stays in `crate::video`, pure scheduling in `crate::engine`.

pub mod delivery_events_stream;
pub mod delivery_types;
pub mod engine_control;
pub mod focus_control;

pub(crate) mod event_snapshots;
pub(crate) mod focus_mapping;
pub(crate) mod runtime_registry;
pub(crate) mod tracked_items;

#[cfg(test)]
mod tests;
