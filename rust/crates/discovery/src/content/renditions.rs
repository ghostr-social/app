//! NIP-71 repeated `imeta` variants mapped at the discovery boundary.

use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DeliveryKind, VideoMeta};
use ghostr_media_model::blossom::terminal_sha256;
use ghostr_media_model::event_identity::VIDEO_KINDS;
use ghostr_media_model::native_media_metadata::NativeMediaMetadata;
use ghostr_media_model::native_models::NativeVideoDelivery;
use ghostr_media_model::nostr_event_media::event_imeta_media;
use nostr_sdk::Event;

pub(crate) fn progressive_renditions(event: &Event) -> Vec<VideoRendition> {
    if !VIDEO_KINDS.contains(&event.kind.as_u16()) {
        return Vec::new();
    }
    event_imeta_media(event)
        .iter()
        .filter_map(progressive_rendition)
        .collect()
}

pub(crate) fn video_meta(media: &NativeMediaMetadata) -> VideoMeta {
    let mut urls = vec![media.url.clone()];
    urls.extend(media.fallback_urls.iter().cloned());
    VideoMeta {
        urls,
        delivery: delivery_kind(media.delivery),
        sha256: media
            .expected_digest
            .clone()
            .or_else(|| terminal_sha256(&media.url)),
        size_bytes: media.extras.size_bytes,
        duration_ms: media.extras.duration_ms,
    }
}

fn progressive_rendition(media: &NativeMediaMetadata) -> Option<VideoRendition> {
    let bitrate = media.extras.bitrate_bps;
    VideoRendition::try_new(video_meta(media), bitrate).ok()
}

fn delivery_kind(delivery: NativeVideoDelivery) -> DeliveryKind {
    match delivery {
        NativeVideoDelivery::Hls => DeliveryKind::Hls,
        NativeVideoDelivery::Progressive => DeliveryKind::Progressive,
    }
}
