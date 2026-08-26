use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use crate::tests::demand_lease_fixture::catalog;
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn deferred_probe_history_does_not_outlive_post_retention() {
    let old = PostId::new("old");
    let kept = PostId::new("kept");
    let catalog = catalog(&["old", "kept"]);
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    assert_eq!(
        probes
            .claim(&catalog, core::slice::from_ref(&old), &retry)
            .len(),
        1
    );
    probes.defer_to_body(&old);

    probes.retain_history(&HashSet::from([kept]));

    assert_eq!(probes.claim(&catalog, &[old], &retry).len(), 1);
}
