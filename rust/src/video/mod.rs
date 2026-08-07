pub mod cache_registry;
mod candidate_priority;
pub mod chunk_cancel;
pub mod chunk_downloader;
mod chunk_network;
mod chunk_response;
mod chunk_stream;
mod content_range;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
mod debug_assets;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod debug_feed;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
mod debug_hls;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
mod debug_http;
pub mod debug_network;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
mod debug_state;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod debug_videos;
mod delivery_cache;
mod delivery_completion;
pub mod delivery_events;
pub mod delivery_failure;
mod delivery_inflight;
pub mod delivery_manager;
mod delivery_plan;
mod delivery_pressure;
mod delivery_probe_completion;
mod delivery_reconcile;
mod delivery_reset;
pub mod delivery_retry;
mod delivery_state;
mod delivery_stats;
mod delivery_transfers;
mod delivery_wake;
mod download_workers;
pub mod event_identity;
pub mod ffi_models;
mod gateway_delivery;
pub mod gateway_runtime;
mod hls_http_gateway;
pub mod hls_manifest;
mod hls_manifest_attributes;
mod hls_manifest_tags;
pub mod hls_playback_gateway;
mod hls_resource_capability;
mod hls_session_state;
mod hls_session_types;
pub mod hls_sessions;
pub mod http_gateway;
pub mod imeta_extras;
pub mod media_probe;
mod metadata_probe_pool;
pub mod mp4_moov;
mod mutable_priority_queue;
mod native_blob_integrity;
mod native_blob_store;
pub mod native_cache;
mod native_cache_capacity;
mod native_cache_digest;
mod native_cache_directory;
mod native_cache_failure;
mod native_cache_fetch;
mod native_cache_transfer;
mod native_download_state;
pub mod native_gateway;
pub mod native_media_metadata;
pub mod native_models;
mod native_partial_store;
mod native_text;
pub mod nostr_event_media;
mod origin_content_type;
pub mod outbound_media_client;
pub mod partial_range_completion;
mod partial_range_disk;
pub mod partial_range_manifest;
mod partial_range_paths;
pub mod partial_range_store;
pub mod playback_demand;
pub mod post_text;
pub mod progressive_posts;
pub mod progressive_route;
mod progressive_stream;
mod public_dns_resolver;
mod public_media_address;
pub mod range_header;
pub mod transfer_timeouts;
pub mod video_link_scan;

#[cfg(test)]
mod tests;

pub use native_gateway as video;
