use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use crate::tests::demand_lease_fixture::catalog;
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn deferred_probe_rearms_when_reconciliation_finds_no_active_body() {
    let post = PostId::new("cancelled");
    let catalog = catalog(&["cancelled"]);
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    assert_eq!(
        probes
            .claim(&catalog, core::slice::from_ref(&post), &retry)
            .len(),
        1
    );
    let identity = catalog
        .transfer_identity(&post, "https://cancelled.example/video.mp4")
        .expect("current identity");
    probes.defer_to_body(&identity);

    probes.reconcile_bodies(&HashSet::new());

    assert_eq!(probes.claim(&catalog, &[post], &retry).len(), 1);
}
