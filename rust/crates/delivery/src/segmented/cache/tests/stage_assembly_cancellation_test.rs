use super::stage_lease_fixture::{focused_cache, object, source, store_partial};
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::prepare_complete;
use ghostr_engine::PostId;
use std::future::{poll_fn, Future};
use std::task::Poll;

const KIB: u64 = 1024;

#[tokio::test]
async fn final_assembly_cancels_at_a_bounded_checkpoint_off_cache_lock() {
    let post = PostId::new("current");
    let cache = focused_cache(&post);
    store_partial(&cache, &post, (1024 * KIB) as usize);
    let request = StageRequest::new(source(), 1024 * KIB, 128 * KIB);
    let fence = StageFence::new(1, 7, request);
    let reservation = StageReservation::final_block(128 * KIB, 1152 * KIB).unwrap();
    let admission = StageAdmission::new(post, fence, 500, reservation);
    let mut lease = cache.admit_stage(admission).expect("final stage admitted");
    let block = object((128 * KIB) as usize);
    let seed = lease.claim_assembly(&block).expect("prefix claimed");
    let (cancel, mut cancelled) = tokio::sync::oneshot::channel();
    let future = prepare_complete(Some(seed), block, &mut cancelled);
    tokio::pin!(future);

    poll_fn(|context| match future.as_mut().poll(context) {
        Poll::Pending => Poll::Ready(()),
        Poll::Ready(_) => panic!("assembly crossed its first cancellation checkpoint"),
    })
    .await;
    cache.clear();
    assert_eq!(cache.physical_used_bytes(), 2304 * KIB);
    cancel.send(()).unwrap();
    assert!(future.await.is_err());

    drop(lease);
    assert_eq!(cache.physical_used_bytes(), 0);
}
