//! FFI contract v1 (plan §2) plus the phase-2 feed/broadcast surface:
//! the narrow surface Dart calls. Only data shapes and thin glue live
//! here — delivery machinery stays in `crate::video`, pure scheduling
//! in `crate::engine`, relay work in `crate::discovery`.

pub(crate) mod accepted_events;
pub mod broadcast_control;
mod candidate_delivery;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod debug_nostr;
pub mod delivery_events_stream;
pub mod delivery_types;
pub mod engine_control;
pub mod event_control;
pub mod event_types;
pub mod feed_control;
pub mod feed_types;
pub mod feed_updates_stream;
pub mod focus_control;
pub mod session_control;

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
mod debug_relay_status;
pub(crate) mod event_runtime;
pub(crate) mod event_snapshots;
pub(crate) mod feed_decisions;
pub(crate) mod feed_mapping;
mod feed_outcome_pump;
pub(crate) mod feed_outcomes;
pub(crate) mod feed_progress;
pub(crate) mod feed_projection;
pub(crate) mod feed_runtime;
mod feed_runtime_start;
pub(crate) mod feed_state;
pub(crate) mod focus_mapping;
pub(crate) mod runtime_configuration;
pub(crate) mod runtime_registry;
pub(crate) mod tracked_items;

#[cfg(test)]
mod tests;
