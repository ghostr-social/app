//! FFI contract v1 (plan §2) plus the phase-2 feed/broadcast surface:
//! the narrow surface Dart calls.
//!
//! The modules in the first block below ARE the contract: their paths
//! are written into `src/frb_generated.rs`, so moving or renaming one
//! means regenerating the bindings and the Dart side with them. Keep
//! them flat and keep them thin.
//!
//! Everything under them is ours to arrange. `runtime` holds the engine
//! between calls, `feed` holds feed state and its mapping into FFI
//! shapes, `delivery` hands media to the delivery engine, and `debug`
//! is the desktop-only web surface. Delivery machinery proper stays in
//! `crate::video`, pure scheduling in `crate::engine`, relay work in
//! `crate::discovery`.

// The generated bindings name these paths. Do not move without
// re-running flutter_rust_bridge_codegen.
pub mod broadcast_control;
pub mod delivery_events_stream;
pub mod delivery_types;
pub mod engine_control;
pub mod event_control;
pub mod event_types;
pub mod feed_control;
pub mod feed_types;
pub mod feed_updates_stream;
pub mod focus_control;
pub mod network_control;
pub mod playback_control;
pub mod playback_preparation_stream;
pub mod playback_types;
pub mod player_preparation_control;
pub mod session_control;

pub(crate) mod delivery;
pub(crate) mod feed;
pub(crate) mod runtime;

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod debug;

#[cfg(test)]
mod tests;
