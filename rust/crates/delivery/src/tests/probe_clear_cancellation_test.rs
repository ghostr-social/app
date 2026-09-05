use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use crate::tests::probe_timeout_identity_fixture::{catalog, MIRROR_A};
use ghostr_engine::PostId;

#[tokio::test]
async fn clearing_focus_aborts_a_probe_but_counts_it_until_terminal_acknowledgement() {
    let post = PostId::new("leaving-feed");
    let catalog = catalog(&post, "first");
    let retry = RetryBook::new(RetryPolicy::default());
    let mut pool = MetadataProbePool::new(1);
    pool.claim_selected(ProbeClaimQuery {
        catalog: &catalog,
        retry: &retry,
        post: &post,
        source: MIRROR_A,
        observed_at_ms: 1,
    })
    .expect("fixture");
    let task = tokio::spawn(core::future::pending::<()>());
    pool.attach_task(&post, task.abort_handle());

    pool.clear();

    assert!(task.await.expect_err("cancelled probe task").is_cancelled());
    assert_eq!(pool.active_identities().len(), 1);
    assert!(pool.current_identity(&catalog, &post, MIRROR_A).is_none());
    pool.release(&post);
    assert!(pool.active_identities().is_empty());
}
