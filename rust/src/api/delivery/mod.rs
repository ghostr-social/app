//! Handing discovered media to the delivery engine, and reading its
//! progress back out as FFI events.

pub(crate) mod candidates;
pub(crate) mod focus_mapping;
pub(crate) mod playback_mapping;
pub(crate) mod snapshot_view;
pub(crate) mod snapshots;
mod snapshots_hls;
