use crate::catalog::Catalog;
use crate::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn integrity_quarantine_revokes_live_delivery_identity() {
    let digest = "d".repeat(64);
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), meta(&digest));
    let identity = binding.transfer(SOURCE).expect("bound source");

    assert_eq!(
        catalog.deliverable_transfer_identity(&post, SOURCE),
        Some(identity.clone())
    );
    catalog.quarantine_source(&identity, &digest, 1);
    assert_eq!(catalog.transfer_identity(&post, SOURCE), Some(identity));
    assert!(catalog
        .deliverable_transfer_identity(&post, SOURCE)
        .is_none());
}

fn meta(digest: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: Some(digest.to_owned()),
        size_bytes: Some(293_999),
        duration_ms: Some(8_000),
    }
}
