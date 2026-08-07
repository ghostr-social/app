//! `ghostr-delivery` — extracted from `rust_lib_ghostr::video`.

pub mod cache_registry;
mod candidate_priority;
pub mod chunk_cancel;
pub mod chunk_downloader;
pub mod chunk_network;
mod chunk_response;
mod chunk_stream;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod debug_feed;
pub mod debug_network;
mod delivery_cache;
mod delivery_completion;
pub mod delivery_events;
pub mod delivery_failure;
pub mod delivery_inflight;
pub mod delivery_manager;
pub mod delivery_plan;
pub mod delivery_pressure;
mod delivery_probe_completion;
pub mod delivery_reconcile;
mod delivery_reset;
pub mod delivery_retry;
pub mod delivery_state;
pub mod delivery_stats;
pub mod delivery_transfers;
mod delivery_wake;
mod download_workers;
pub mod media_probe;
pub mod metadata_probe_pool;
pub mod mp4_moov;
pub mod mutable_priority_queue;
pub mod playback_demand;
pub mod progressive_posts;

#[cfg(test)]
mod tests;
