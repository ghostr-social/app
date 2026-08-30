use crate::probe::pool::MetadataProbePool;
use crate::tests::probe_timeout_identity_fixture::{catalog, identity, MIRROR_A, MIRROR_B};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn another_mirror_finishing_keeps_the_timed_out_mirror_marker() {
    let post = PostId::new("mirrors");
    let catalog = catalog(&post, "first");
    let mirror_a = identity(&catalog, &post, MIRROR_A);
    let mirror_b = identity(&catalog, &post, MIRROR_B);
    let mut probes = MetadataProbePool::new(1);
    probes.require_body(&mirror_a);

    probes.body_finished(&mirror_b);

    assert_eq!(
        probes.current_unavailable_identities(&catalog),
        HashSet::from([mirror_a])
    );
}
