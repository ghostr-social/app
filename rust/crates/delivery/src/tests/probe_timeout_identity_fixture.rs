use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

pub const MIRROR_A: &str = "https://a.example/video.mp4";
pub const MIRROR_B: &str = "https://b.example/video.mp4";

pub fn catalog(post: &PostId, digest: &str) -> Catalog {
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), metadata(digest));
    catalog
}

pub fn identity(catalog: &Catalog, post: &PostId, source: &str) -> TransferIdentity {
    catalog
        .transfer_identity(post, source)
        .expect("current transfer identity")
}

pub fn metadata(digest: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![MIRROR_A.into(), MIRROR_B.into()],
        delivery: DeliveryKind::Progressive,
        sha256: Some(digest.into()),
        size_bytes: None,
        duration_ms: None,
    }
}
