use super::{ResourceObservation, ResourcePrices, ShadowPriceController};

#[test]
fn repeated_updates_equal_literal_controller_updates() {
    let actual = ResourceObservation::new(180, 70, 12, 5);
    let target = ResourceObservation::new(100, 100, 10, 8);
    for count in [0, 1, 7, 1_000_000] {
        let mut literal = controller();
        let mut repeated = controller();
        for _ in 0..count {
            literal.observe(actual, target);
        }
        repeated.observe_repeated(actual, target, count);
        assert_eq!(repeated.prices(), literal.prices());
    }
}

#[test]
fn repeated_updates_preserve_saturation_and_zero_cpu_target() {
    let prices = ResourcePrices {
        network_micros: u64::MAX - 2,
        storage_micros: 2,
        cpu_micros: 77,
        request_micros: u64::MAX - 2,
    };
    let mut controller = ShadowPriceController::from_prices(prices);
    controller.observe_repeated(
        ResourceObservation::new(u64::MAX, 0, u64::MAX, u64::MAX),
        ResourceObservation::new(1, u64::MAX, 0, 1),
        u128::MAX,
    );

    assert_eq!(controller.prices().network_micros, u64::MAX);
    assert_eq!(controller.prices().storage_micros, 0);
    assert_eq!(controller.prices().cpu_micros, 77);
    assert_eq!(controller.prices().request_micros, u64::MAX);
}

#[test]
fn single_update_uses_the_exact_ratio_at_maximum_target() {
    let mut controller = controller();

    controller.observe_storage(0, u64::MAX);

    assert_eq!(controller.prices().storage_micros, 3_000);
}

fn controller() -> ShadowPriceController {
    ShadowPriceController::from_prices(ResourcePrices {
        network_micros: 4_000,
        storage_micros: 4_000,
        cpu_micros: 4_000,
        request_micros: 4_000,
    })
}
