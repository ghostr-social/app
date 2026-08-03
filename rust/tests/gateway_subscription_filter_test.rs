use nostr_sdk::Kind;
use rust_lib_ghostr::video::gateway_runtime::{
    deletion_filter, video_filter, MAX_NATIVE_INVENTORY_ITEMS,
};

#[test]
fn limits_the_initial_relay_query_to_the_native_inventory_capacity() {
    let filter = video_filter();

    assert_eq!(filter.limit, Some(MAX_NATIVE_INVENTORY_ITEMS));
}

#[test]
fn subscribes_to_a_bounded_deletion_stream() {
    let filter = deletion_filter();

    assert_eq!(filter.limit, Some(MAX_NATIVE_INVENTORY_ITEMS));
    assert!(filter
        .kinds
        .as_ref()
        .is_some_and(|kinds| kinds.contains(&Kind::EventDeletion)));
}
