use crate::api::delivery_types::FfiMediaDelivery;
use crate::api::delivery::focus_mapping::focus_item;
use crate::api::tests::support::ffi_item;
use crate::engine::DeliveryKind;

#[test]
fn carries_the_discovery_metadata_into_the_engine_item() {
    let item = ffi_item("clip", FfiMediaDelivery::Progressive);

    let mapped = focus_item(&item).expect("mapped item");

    assert_eq!(mapped.post.as_str(), "clip");
    assert_eq!(mapped.meta.urls, item.urls);
    assert_eq!(mapped.meta.delivery, DeliveryKind::Progressive);
    assert_eq!(mapped.meta.sha256, item.sha256);
    assert_eq!(mapped.meta.size_bytes, Some(16));
    assert_eq!(mapped.meta.duration_ms, Some(2_000));
}

#[test]
fn rejects_an_item_with_an_unsafe_post_id() {
    let item = ffi_item("../escape", FfiMediaDelivery::Progressive);

    assert!(focus_item(&item).is_err());
}
