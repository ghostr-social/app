use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use crate::tests::demand_lease_fixture::catalog;
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn expired_head_requires_a_successful_matching_body() {
    let post = PostId::new("body-required");
    let catalog = catalog(&["body-required"]);
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    assert_eq!(
        probes
            .claim(&catalog, core::slice::from_ref(&post), &retry)
            .len(),
        1
    );
    let identity = catalog
        .transfer_identity(&post, "https://body-required.example/video.mp4")
        .expect("current identity");

    probes.require_body(&identity);
    probes.reconcile_bodies(&HashSet::new());

    assert_eq!(
        probes.current_unavailable_identities(&catalog),
        HashSet::from([identity.clone()])
    );
    assert!(probes
        .claim(&catalog, core::slice::from_ref(&post), &retry)
        .is_empty());
    probes.body_finished(&identity);
    assert_eq!(
        probes.current_unavailable_identities(&catalog),
        HashSet::from([identity.clone()])
    );
    assert!(probes
        .claim(&catalog, core::slice::from_ref(&post), &retry)
        .is_empty());
    probes.body_satisfied(&identity);
    probes.body_finished(&identity);
    assert_eq!(probes.claim(&catalog, &[post], &retry).len(), 1);
}
