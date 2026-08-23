use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::collections::HashSet;

#[test]
fn evicted_probe_history_is_dropped_without_reprobing_retained_posts() {
    let old = PostId::new("old");
    let kept = PostId::new("kept");
    let mut catalog = Catalog::new();
    catalog.upsert(old.clone(), unknown_meta("old"));
    catalog.upsert(kept.clone(), unknown_meta("kept"));
    let mut probes = MetadataProbePool::new(2);
    probes.learned(
        &catalog.transfer_identity(&old, &unknown_meta("old").urls[0]).unwrap(),
        None,
    );
    probes.learned(
        &catalog.transfer_identity(&kept, &unknown_meta("kept").urls[0]).unwrap(),
        None,
    );

    probes.retain_history(&HashSet::from([kept.clone()]));
    let retry = RetryBook::new(RetryPolicy::default());
    let claimed = probes.claim(&catalog, &[old.clone(), kept], &retry);

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].0, old);
}

fn unknown_meta(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
