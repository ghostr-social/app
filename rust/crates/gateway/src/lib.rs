//! `ghostr-gateway` — extracted from `rust_lib_ghostr::video`.

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
mod debug_hls;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
mod debug_http;
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
mod gateway_delivery;
pub mod gateway_runtime;
mod hls_http_gateway;
pub mod hls_playback_gateway;
mod hls_resource_capability;
mod hls_session_state;
mod hls_session_types;
pub mod hls_sessions;
pub mod http_gateway;
pub mod progressive_route;
mod progressive_stream;
pub mod range_header;
