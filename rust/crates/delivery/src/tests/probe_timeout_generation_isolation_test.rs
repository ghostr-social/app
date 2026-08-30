use crate::probe::pool::MetadataProbePool;
use crate::tests::probe_timeout_identity_fixture::{catalog, identity, metadata, MIRROR_A};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn stale_generation_finishing_keeps_the_current_generation_marker() {
    let post = PostId::new("generation");
    let mut catalog = catalog(&post, "first");
    let stale = identity(&catalog, &post, MIRROR_A);
    catalog.upsert(post.clone(), metadata("replacement"));
    let current = identity(&catalog, &post, MIRROR_A);
    let mut probes = MetadataProbePool::new(1);
    probes.require_body(&current);

    probes.body_finished(&stale);

    assert_eq!(
        probes.current_unavailable_identities(&catalog),
        HashSet::from([current])
    );
}
