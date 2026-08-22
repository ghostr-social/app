use ghostr_engine::evidence::NostrMetadataEvidence;
use ghostr_media_model::native_media_metadata::NativeMediaMetadata;
use ghostr_media_model::nostr_event_media::{event_imeta_media, tag_values};
use nostr_sdk::Event;

pub(super) fn metadata(
    event: &Event,
    selected: &NativeMediaMetadata,
) -> Vec<NostrMetadataEvidence> {
    let mut variants = event_imeta_media(event);
    if variants.is_empty() {
        variants.push(selected.clone());
    }
    variants
        .into_iter()
        .map(|media| evidence(event, media))
        .collect()
}

fn evidence(event: &Event, media: NativeMediaMetadata) -> NostrMetadataEvidence {
    let mut urls = vec![media.url];
    urls.extend(media.fallback_urls);
    NostrMetadataEvidence {
        issuer: event.pubkey.to_hex(),
        client: client(event),
        event_id: event.id.to_hex(),
        observed_at_ms: event.created_at.as_u64().saturating_mul(1_000),
        urls,
        mime: media.declared_mime,
        size_bytes: media.extras.size_bytes,
        duration_ms: media.extras.duration_ms,
        dimensions: media.extras.dimensions,
        bitrate_bps: media.extras.bitrate_bps,
        sha256: media.expected_digest,
        original_sha256: media.original_digest,
    }
}

fn client(event: &Event) -> Option<String> {
    let value = tag_values(event, "client").next()?.trim();
    (!value.is_empty()).then(|| value.chars().take(128).collect())
}
