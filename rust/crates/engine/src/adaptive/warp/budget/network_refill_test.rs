use super::NetworkTokenBucket;

#[test]
fn refill_deadline_uses_the_exact_token_deficit() {
    let mut bucket = NetworkTokenBucket::new(100, 20, 1_000);
    assert!(bucket.consume(90, 1_000));

    assert_eq!(bucket.refill_deadline_ms(50, 1_000), Some(3_000));
    assert_eq!(bucket.refill_deadline_ms(10, 1_000), None);
    assert_eq!(bucket.refill_deadline_ms(101, 1_000), None);
}

#[test]
fn fractional_millisecond_refill_rounds_up_to_a_safe_deadline() {
    let mut bucket = NetworkTokenBucket::new(10, 3_000, 50);
    assert!(bucket.consume(10, 50));

    assert_eq!(bucket.refill_deadline_ms(1, 50), Some(51));
}

#[test]
fn repeated_sub_token_polls_preserve_fractional_refill_credit() {
    let mut stepped = empty_bucket();
    for now_ms in [1_010, 1_020, 1_030, 1_040] {
        assert_eq!(stepped.available(now_ms), 0);
        assert_eq!(stepped.refill_deadline_ms(1, now_ms), Some(1_050));
    }
    let mut single = empty_bucket();

    assert_eq!(stepped.available(1_050), 1);
    assert_eq!(stepped.available(1_050), single.available(1_050));
}

fn empty_bucket() -> NetworkTokenBucket {
    let mut bucket = NetworkTokenBucket::new(10, 20, 1_000);
    assert!(bucket.consume(10, 1_000));
    bucket
}

#[test]
fn fractional_credit_survives_the_planner_replay_state() {
    let mut bucket = NetworkTokenBucket::from_replay((10, 20, 0, 1_000, 800, 0));

    assert_eq!(bucket.replay_parts(), (10, 20, 0, 1_000, 800, 0));
    assert_eq!(bucket.refill_deadline_ms(1, 1_000), Some(1_010));
}

#[test]
fn terminal_actual_bytes_credit_unused_reservation_and_charge_overrun() {
    let mut bucket = NetworkTokenBucket::new(100, 0, 1_000);
    assert!(bucket.consume(100, 1_000));

    bucket.reconcile_reservation(100, 10, 1_000);
    assert_eq!(bucket.available(1_000), 90);
    bucket.reconcile_reservation(10, 20, 1_000);
    assert_eq!(bucket.available(1_000), 80);
}

#[test]
fn overrun_debt_is_repaid_before_refill_becomes_available() {
    let mut bucket = NetworkTokenBucket::new(10, 20, 1_000);
    assert!(bucket.consume(10, 1_000));
    bucket.reconcile_reservation(10, 15, 1_000);
    let mut restored = NetworkTokenBucket::from_replay(bucket.replay_parts());

    assert_eq!(restored.available(1_249), 0);
    assert_eq!(restored.available(1_250), 0);
    assert_eq!(restored.available(1_300), 1);
}

#[test]
fn capacity_reconfiguration_preserves_consumed_network_credit() {
    let mut bucket = NetworkTokenBucket::new(8, 0, 1_000);
    assert!(bucket.consume(1, 1_000));

    bucket.reconfigure(1, 0, 1_000);
    assert_eq!(bucket.available(1_000), 0);
    bucket.reconfigure(8, 0, 1_000);
    assert_eq!(bucket.available(1_000), 7);

    bucket.reconcile_reservation(1, 0, 1_000);
    assert_eq!(bucket.available(1_000), 8);
}

#[test]
fn reservation_refund_repays_existing_overrun_debt_first() {
    let mut bucket = NetworkTokenBucket::new(100, 0, 1_000);
    assert!(bucket.consume(100, 1_000));

    bucket.reconcile_reservation(50, 60, 1_000);
    bucket.reconcile_reservation(50, 0, 1_000);

    assert_eq!(bucket.available(1_000), 40);
    assert!(!bucket.consume(41, 1_000));
}
