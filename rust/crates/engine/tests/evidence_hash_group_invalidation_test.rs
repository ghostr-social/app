use ghostr_engine::catalog::{Catalog, CatalogEvidenceState};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn digest_mismatch_quarantines_only_the_exact_advertised_hash_group() {
    let digest = "a".repeat(64);
    let mut catalog = Catalog::new();
    let failed = PostId::new("failed");
    let peer = PostId::new("peer");
    let cached_peer = PostId::new("cached-peer");
    let lineage_only = PostId::new("lineage");
    let unhashed = PostId::new("unhashed");
    let binding = catalog.upsert(failed.clone(), meta("failed", Some(&digest)));
    catalog.upsert(peer.clone(), meta("peer", Some(&digest)));
    catalog.upsert(cached_peer.clone(), meta("cached", Some(&digest)));
    catalog.upsert(lineage_only.clone(), meta("lineage", Some(&"b".repeat(64))));
    catalog.upsert(unhashed.clone(), meta("unhashed", None));
    catalog.retain(|post| post != &cached_peer);
    let identity = binding
        .transfer("https://failed.example/video.mp4")
        .unwrap();
    let unrelated = "e".repeat(64);
    assert!(catalog
        .quarantine_mirror_group(&identity, &unrelated, 19)
        .is_empty());
    let unrelated_post = PostId::new("unrelated");
    catalog.upsert(unrelated_post.clone(), meta("unrelated", Some(&unrelated)));
    assert!(!catalog.lookup(&unrelated_post).unwrap().is_quarantined());

    let invalidated = catalog.quarantine_mirror_group(&identity, &digest, 20);
    assert_eq!(invalidated, vec![cached_peer, failed.clone(), peer.clone()]);
    assert!(catalog.lookup(&failed).unwrap().is_quarantined());
    assert!(catalog.lookup(&peer).unwrap().is_quarantined());
    assert!(!catalog.lookup(&lineage_only).unwrap().is_quarantined());
    assert!(!catalog.lookup(&unhashed).unwrap().is_quarantined());
    let late = PostId::new("late");
    catalog.upsert(late.clone(), meta("late", Some(&digest)));
    assert!(catalog.lookup(&late).unwrap().is_quarantined());
}

#[test]
fn exact_hash_claims_and_quarantine_survive_catalog_restart() {
    let digest = "c".repeat(64);
    let cached = PostId::new("cached");
    let mut before = Catalog::new();
    before.upsert(cached.clone(), meta("cached", Some(&digest)));
    let json = before.evidence_state().to_json();

    let mut after = Catalog::new();
    after.replace_evidence_state(CatalogEvidenceState::from_json(&json).unwrap(), 1);
    let failed = PostId::new("failed");
    let binding = after.upsert(failed.clone(), meta("failed", Some(&digest)));
    let identity = binding
        .transfer("https://failed.example/video.mp4")
        .unwrap();

    assert_eq!(
        after.quarantine_mirror_group(&identity, &digest, 2),
        vec![cached, failed]
    );
    let restored = CatalogEvidenceState::from_json(&after.evidence_state().to_json()).unwrap();
    let mut restarted = Catalog::new();
    restarted.replace_evidence_state(restored, 3);
    let late = PostId::new("late");
    restarted.upsert(late.clone(), meta("late", Some(&digest)));
    assert!(restarted.lookup(&late).unwrap().is_quarantined());
}

fn meta(host: &str, sha256: Option<&str>) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{host}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: sha256.map(str::to_owned),
        size_bytes: Some(100),
        duration_ms: Some(1_000),
    }
}
