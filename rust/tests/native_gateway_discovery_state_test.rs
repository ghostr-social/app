use rust_lib_ghostr::video::video::ffi_get_discovered_videos;

#[tokio::test]
async fn returns_an_empty_inventory_before_the_gateway_starts() {
    assert!(ffi_get_discovered_videos().await.is_empty());
}
