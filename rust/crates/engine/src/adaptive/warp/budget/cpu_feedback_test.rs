use super::{ResourceObservation, ShadowPriceController};

#[test]
fn absent_cpu_sample_preserves_the_learned_price() {
    let mut controller = ShadowPriceController::default();
    controller.observe(cpu(2), cpu(1));
    let learned = controller.prices().cpu_micros;
    assert!(learned > 0);

    controller.observe(cpu(0), cpu(0));

    assert_eq!(controller.prices().cpu_micros, learned);
}

#[test]
fn near_limit_cpu_sample_raises_price_against_operating_target() {
    let target = ShadowPriceController::cpu_operating_target_ms(10);
    assert_eq!(target, 9);
    let mut controller = ShadowPriceController::default();

    controller.observe(cpu(10), cpu(target));

    assert!(controller.prices().cpu_micros > 0);
}

#[test]
fn absent_cpu_semantics_do_not_disable_zero_network_budget_pressure() {
    let mut controller = ShadowPriceController::default();
    controller.observe(
        ResourceObservation::new(2, 0, 0, 0),
        ResourceObservation::default(),
    );

    assert!(controller.prices().network_micros > 0);
}

fn cpu(cpu_ms: u64) -> ResourceObservation {
    ResourceObservation::new(0, 0, cpu_ms, 0)
}
