//! Documented contract choice: HLS items never get a progressive URL.
//! HLS playback stays on the session-owning ffi_acquire_hls_playback.

use rust_lib_ghostr::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use rust_lib_ghostr::api::focus_control::ffi_playback_url;

#[tokio::test]
async fn refuses_a_progressive_url_for_hls_items() {
    let item = FfiFocusItem {
        post_id: "stream".to_owned(),
        urls: vec!["https://media.example/stream.m3u8".to_owned()],
        delivery: FfiMediaDelivery::Hls,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    };

    let error = ffi_playback_url(item).await.expect_err("hls rejection");

    assert!(error.to_string().contains("ffi_acquire_hls_playback"));
}
