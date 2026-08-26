use super::*;

impl CachedHlsObject {
    pub fn new(body: Arc<[u8]>, final_url: Url, content_type: Option<String>) -> Self {
        Self::with_metadata(body, final_url, content_type, HlsCacheMetadata::default())
    }

    pub(in crate::segmented) fn with_metadata(
        body: Arc<[u8]>,
        final_url: Url,
        content_type: Option<String>,
        metadata: HlsCacheMetadata,
    ) -> Self {
        let generation =
            CachedHlsGeneration::for_object(&final_url, body.as_ref(), metadata.validator.as_ref());
        Self {
            body,
            final_url,
            content_type,
            validator: metadata.validator,
            fresh_until: metadata.fresh_until,
            generation,
        }
    }
}
