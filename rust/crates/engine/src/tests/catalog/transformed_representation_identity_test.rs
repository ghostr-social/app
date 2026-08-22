use crate::adaptive::TransformKind;
use crate::catalog::Catalog;
use crate::{DeliveryKind, PostId, VideoMeta};

#[test]
fn transformed_identity_is_exactly_derived_and_has_no_remote_source() {
    let post = PostId::new("post");
    let source = "https://origin.example/video.mp4";
    let mut catalog = Catalog::new();
    let input = catalog.upsert(
        post,
        VideoMeta {
            urls: vec![source.into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(24),
            duration_ms: Some(1_000),
        },
    );
    let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let output = input
        .derive_transform(TransformKind::Remux, digest)
        .expect("valid derived identity");

    assert!(output.derives_from(&input));
    assert_ne!(output.representation(), input.representation());
    assert!(output.transfer(source).is_none());
    assert!(input
        .derive_transform(TransformKind::Remux, "not-a-digest")
        .is_none());
}
