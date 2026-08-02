mod support;

use rust_lib_ghostr::video::ffi_models::{ffi_video_download, FfiVideoDelivery};
use rust_lib_ghostr::video::native_models::NativeVideoDelivery;
use support::fixtures::native_download;

#[test]
fn preserves_hls_delivery_across_the_ffi_boundary() {
    let mut native = native_download("https://media.example/video.m3u8");
    native.nostr.delivery = NativeVideoDelivery::Hls;

    let ffi = ffi_video_download(&native);

    assert_eq!(ffi.nostr.delivery, FfiVideoDelivery::Hls);
}
