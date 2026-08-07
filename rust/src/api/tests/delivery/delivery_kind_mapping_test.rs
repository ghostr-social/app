use crate::api::delivery_types::FfiMediaDelivery;
use crate::engine::DeliveryKind;

#[test]
fn maps_the_two_delivery_kinds() {
    assert_eq!(
        DeliveryKind::from(FfiMediaDelivery::Progressive),
        DeliveryKind::Progressive
    );
    assert_eq!(DeliveryKind::from(FfiMediaDelivery::Hls), DeliveryKind::Hls);
}
