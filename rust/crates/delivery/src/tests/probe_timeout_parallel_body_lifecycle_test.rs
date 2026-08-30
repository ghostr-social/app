use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use crate::tests::probe_timeout_identity_fixture::{catalog, identity, MIRROR_A};
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn successful_body_waits_for_parallel_body_exit_before_rearming_head() {
    let post = PostId::new("parallel-body");
    let catalog = catalog(&post, "first");
    let identity = identity(&catalog, &post, MIRROR_A);
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    probes.require_body(&identity);
    probes.body_satisfied(&identity);

    probes.reconcile_bodies(&HashSet::from([identity.clone()]));
    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &post)),
        Err(ProbeClaimRefusal::DeferredToBody)
    );
    probes.reconcile_bodies(&HashSet::new());
    assert!(probes
        .claim_selected(query(&catalog, &retry, &post))
        .is_ok());
}

fn query<'a>(
    catalog: &'a ghostr_engine::catalog::Catalog,
    retry: &'a RetryBook,
    post: &'a PostId,
) -> ProbeClaimQuery<'a> {
    ProbeClaimQuery {
        catalog,
        retry,
        post,
        source: MIRROR_A,
        observed_at_ms: 1,
    }
}
