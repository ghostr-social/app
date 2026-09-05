use crate::catalog::{Catalog, CatalogEvidenceState};
use crate::{DeliveryKind, PostId, VideoMeta};

const BAD: &str = "https://bad.example/video.mp4";
const GOOD: &str = "https://good.example/video.mp4";

#[test]
fn one_bad_mirror_cannot_poison_healthy_sources_or_other_posts_after_restart() {
    let digest = "d".repeat(64);
    let failed = PostId::new("failed");
    let healthy = PostId::new("healthy");
    let meta = VideoMeta {
        urls: vec![BAD.into(), GOOD.into()],
        delivery: DeliveryKind::Progressive,
        sha256: Some(digest.clone()),
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    };
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(failed.clone(), meta.clone());
    catalog.upsert(healthy.clone(), meta.clone());
    let identity = binding.transfer(BAD).expect("bound bad source");
    assert_eq!(
        catalog.quarantine_source(&identity, &digest, 1),
        vec![failed.clone()]
    );
    let persisted = CatalogEvidenceState::from_json(&catalog.evidence_state().to_json())
        .expect("persisted provenance");
    let mut restored = Catalog::new();
    restored.replace_evidence_state(persisted, 2);
    restored.upsert(failed.clone(), meta.clone());
    restored.upsert(healthy.clone(), meta);

    for state in [catalog, restored] {
        assert!(state.deliverable_transfer_identity(&failed, BAD).is_none());
        assert!(
            state.deliverable_transfer_identity(&failed, GOOD).is_some(),
            "healthy fallback survives"
        );
        assert!(
            state
                .deliverable_transfer_identity(&healthy, GOOD)
                .is_some(),
            "same-x claim is not bad provenance"
        );
    }
}
