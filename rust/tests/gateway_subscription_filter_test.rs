use rust_lib_ghostr::video::gateway_runtime::{video_filter, MAX_NATIVE_INVENTORY_ITEMS};

#[test]
fn limits_the_initial_relay_query_to_the_native_inventory_capacity() {
    let filter = video_filter();

    assert_eq!(filter.limit, Some(MAX_NATIVE_INVENTORY_ITEMS));
}
