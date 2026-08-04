use crate::api::focus_mapping::parse_delivery_kind;
use crate::engine::DeliveryKind;

#[test]
fn maps_the_two_delivery_kinds() {
    assert_eq!(
        parse_delivery_kind("progressive").expect("progressive"),
        DeliveryKind::Progressive
    );
    assert_eq!(parse_delivery_kind("hls").expect("hls"), DeliveryKind::Hls);
}

#[test]
fn rejects_an_unknown_delivery_kind() {
    let error = parse_delivery_kind("torrent").expect_err("unknown kind");

    assert!(error.to_string().contains("torrent"));
}
