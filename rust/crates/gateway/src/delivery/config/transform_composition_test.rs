use super::production_transform;

#[test]
fn unavailable_thread_cpu_clock_suppresses_production_transform_backend() {
    assert!(production_transform(false).is_none());
}
