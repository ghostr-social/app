use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use crate::tests::probe_timeout_identity_fixture::{catalog, identity, MIRROR_A, MIRROR_B};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn active_mirror_does_not_retain_another_mirrors_deferral() {
    let post = PostId::new("reconcile-mirrors");
    let catalog = catalog(&post, "first");
    let mirror_a = identity(&catalog, &post, MIRROR_A);
    let mirror_b = identity(&catalog, &post, MIRROR_B);
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    probes.defer_to_body(&mirror_a);

    probes.reconcile_bodies(&HashSet::from([mirror_b]));

    assert!(probes
        .claim_selected(ProbeClaimQuery {
            catalog: &catalog,
            retry: &retry,
            post: &post,
            source: MIRROR_A,
            observed_at_ms: 1,
        })
        .is_ok());
}
