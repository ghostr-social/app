use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use crate::tests::demand_lease_fixture::catalog;
use ghostr_engine::PostId;

#[test]
fn deferred_probe_rearms_only_after_the_matching_body_finishes() {
    let post = PostId::new("body");
    let catalog = catalog(&["body"]);
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);

    assert_eq!(
        probes
            .claim(&catalog, core::slice::from_ref(&post), &retry)
            .len(),
        1
    );
    let identity = catalog
        .transfer_identity(&post, "https://body.example/video.mp4")
        .expect("current identity");
    probes.defer_to_body(&identity);
    assert!(probes
        .claim(&catalog, core::slice::from_ref(&post), &retry)
        .is_empty());

    probes.body_finished(&identity);
    assert_eq!(probes.claim(&catalog, &[post], &retry).len(), 1);
}
