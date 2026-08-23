use super::stage_capacity_fixture::{cache_with_ready_bytes, final_admission, object, MIB};
use crate::segmented::prepare::prepare_complete;

#[tokio::test]
async fn completed_assembly_reservation_stays_counted_until_lease_release() {
    let (cache, post) = cache_with_ready_bytes(16 * MIB, 4 * MIB);
    let mut lease = cache
        .admit_stage(final_admission(&post, 4 * MIB, 4 * MIB, 8 * MIB))
        .expect("final assembly admitted");
    let block = object("current", 4 * MIB);
    let seed = lease.claim_assembly(&block).expect("prefix claimed");
    let (_cancel, mut cancelled) = tokio::sync::oneshot::channel();
    let completed = prepare_complete(Some(seed), block, &mut cancelled)
        .await
        .expect("assembly completes");

    assert_eq!(cache.physical_used_bytes(), 32 * MIB as u64);
    drop(completed);
    assert_eq!(cache.physical_used_bytes(), 32 * MIB as u64);
    drop(lease);
    assert_eq!(cache.physical_available_bytes(), 12 * MIB as u64);
}
