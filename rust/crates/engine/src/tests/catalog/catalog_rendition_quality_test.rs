use crate::adaptive::PlannerQuality;
use crate::catalog::Catalog;
use crate::video_rendition::VideoRendition;
use crate::{DeliveryKind, PostId, VideoMeta};

#[test]
fn complete_explicit_ladder_yields_normalized_active_quality() {
    let post = PostId::new("quality");
    let low = rendition("low", Some(1_000_000));
    let high = rendition("high", Some(6_000_000));
    let mut catalog = Catalog::new();
    catalog.upsert_with_renditions(post.clone(), low.meta().clone(), vec![low, high]);

    let quality = catalog.rendition_quality(&post).expect("exact ladder");

    assert_eq!(quality.active_bitrate_bps(), 1_000_000);
    assert_eq!(quality.ceiling_bitrate_bps(), 6_000_000);
    assert_eq!(quality.normalized_micros(), 166_666);
    assert_eq!(
        PlannerQuality::from_rendition(quality),
        PlannerQuality::Estimated {
            expected_micros: 166_666,
            lower_micros: 166_666,
            uncertainty_bps: 0,
        }
    );
}

#[test]
fn incomplete_or_unrated_ladder_cannot_invent_quality() {
    let post = PostId::new("unrated");
    let low = rendition("low", Some(1_000_000));
    let mut catalog = Catalog::new();
    catalog.upsert_with_renditions(
        post.clone(),
        low.meta().clone(),
        vec![low, rendition("unknown", None)],
    );
    assert!(catalog.rendition_quality(&post).is_none());

    let plain = PostId::new("plain");
    catalog.upsert(plain.clone(), meta("plain"));
    assert!(catalog.rendition_quality(&plain).is_none());

    let duplicate = PostId::new("duplicate");
    let first = rendition("first", Some(1_000_000));
    catalog.upsert_with_renditions(
        duplicate.clone(),
        first.meta().clone(),
        vec![first, rendition("second", Some(1_000_000))],
    );
    assert!(catalog.rendition_quality(&duplicate).is_none());
}

fn rendition(name: &str, bitrate: Option<u64>) -> VideoRendition {
    VideoRendition::try_new(meta(name), bitrate).expect("valid test fixture")
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{name}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(format!("{name}-digest")),
        size_bytes: Some(1_000_000),
        duration_ms: Some(8_000),
    }
}
