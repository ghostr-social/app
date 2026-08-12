use crate::video_rendition::{VideoRendition, VideoRenditionError};
use crate::{DeliveryKind, VideoMeta};

#[test]
fn video_rendition_rejects_invalid_boundaries_and_keeps_optional_bitrate() {
    assert_eq!(
        VideoRendition::try_new(meta(Vec::new(), DeliveryKind::Progressive), Some(1)),
        Err(VideoRenditionError::EmptySources)
    );
    assert_eq!(
        VideoRendition::try_new(meta(vec![url()], DeliveryKind::Hls), Some(1)),
        Err(VideoRenditionError::NotProgressive)
    );
    assert_eq!(
        VideoRendition::try_new(meta(vec![url()], DeliveryKind::Progressive), Some(0)),
        Err(VideoRenditionError::ZeroBitrate)
    );

    let unrated = VideoRendition::try_new(meta(vec![url()], DeliveryKind::Progressive), None)
        .expect("bitrate is recommended but optional in NIP-71");
    assert_eq!(unrated.meta().urls, [url()]);
    assert_eq!(unrated.bitrate_bits_per_second(), None);
    assert!(unrated.quality().is_none());
    assert_eq!(
        unrated.quality_id().as_str(),
        unrated.identity().fingerprint()
    );
}

fn meta(urls: Vec<String>, delivery: DeliveryKind) -> VideoMeta {
    VideoMeta {
        urls,
        delivery,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}

fn url() -> String {
    "https://cdn.example/video.mp4".to_owned()
}
