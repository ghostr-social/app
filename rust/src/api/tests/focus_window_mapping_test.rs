use crate::api::delivery_types::FfiMediaDelivery;
use crate::api::focus_mapping::delivery_focus;
use crate::api::tests::support::ffi_item;

#[test]
fn maps_the_window_position_and_watch_time() {
    let items = [
        ffi_item("first", FfiMediaDelivery::Progressive),
        ffi_item("second", FfiMediaDelivery::Hls),
    ];

    let focus = delivery_focus(&items, 1, 2_500).expect("mapped focus");

    assert_eq!(focus.items.len(), 2);
    assert_eq!(focus.items[1].post.as_str(), "second");
    assert_eq!(focus.current_index, 1);
    assert_eq!(focus.watch_ms, 2_500);
}
