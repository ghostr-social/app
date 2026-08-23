use super::stage_capacity_fixture::{cache_with_ready_bytes, final_admission, MIB};

#[test]
fn final_assembly_is_rejected_before_exceeding_the_cache_limit() {
    let (cache, post) = cache_with_ready_bytes(16 * MIB, 8 * MIB);
    let admission = final_admission(&post, 8 * MIB, 4 * MIB, 12 * MIB);

    assert!(cache.admit_stage(admission).is_none());
    assert_eq!(cache.physical_available_bytes(), 8 * MIB as u64);
}
