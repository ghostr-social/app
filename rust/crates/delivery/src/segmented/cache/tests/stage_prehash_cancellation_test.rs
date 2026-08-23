use super::stage_lease_fixture::object;
use crate::segmented::prepare::prepare_complete;
use std::future::{poll_fn, Future};
use std::task::Poll;

const MIB: usize = 1024 * 1024;

#[tokio::test]
async fn first_block_hashing_cancels_at_a_bounded_checkpoint() {
    let (cancel, mut cancelled) = tokio::sync::oneshot::channel();
    let future = prepare_complete(None, object(MIB), &mut cancelled);
    tokio::pin!(future);

    poll_fn(|context| match future.as_mut().poll(context) {
        Poll::Pending => Poll::Ready(()),
        Poll::Ready(_) => panic!("prehash crossed its first cancellation checkpoint"),
    })
    .await;
    cancel.send(()).unwrap();

    assert!(future.await.is_err());
}
