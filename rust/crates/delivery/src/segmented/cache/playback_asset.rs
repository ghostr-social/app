use super::{CachedHlsObject, HlsPreparedAssetAuthority, SegmentedCache, SegmentedPhase};

#[derive(Clone)]
pub struct PreparedHlsPlaybackAsset {
    authority: HlsPreparedAssetAuthority,
    root_source: String,
    objects: Vec<PreparedHlsPlaybackObject>,
}

#[derive(Clone)]
struct PreparedHlsPlaybackObject {
    request_url: String,
    object: CachedHlsObject,
}

impl PreparedHlsPlaybackAsset {
    pub fn authority(&self) -> &HlsPreparedAssetAuthority {
        &self.authority
    }

    pub fn root_source(&self) -> &str {
        &self.root_source
    }

    pub fn object(&self, url: &str) -> Option<CachedHlsObject> {
        self.objects
            .iter()
            .find(|known| known.matches(url))
            .map(|known| known.object.clone())
    }
}

impl PreparedHlsPlaybackObject {
    fn matches(&self, url: &str) -> bool {
        let canonical = super::super::source_key::canonical;
        self.request_url == url
            || self.object.final_url.as_str() == url
            || canonical(&self.request_url) == canonical(url)
            || canonical(self.object.final_url.as_str()) == canonical(url)
    }
}

impl SegmentedCache {
    pub fn capture_prepared_asset(
        &self,
        authority: &HlsPreparedAssetAuthority,
        sources: &[String],
    ) -> Option<PreparedHlsPlaybackAsset> {
        let state = self.lock();
        let record = state.focus.get(authority.post())?;
        if record.snapshot.phase != SegmentedPhase::Ready
            || record.snapshot.authority.as_ref() != Some(authority)
            || record.sources != sources
        {
            return None;
        }
        let objects = record
            .objects
            .iter()
            .map(|key| {
                Some(PreparedHlsPlaybackObject {
                    request_url: key.clone(),
                    object: state.objects.get(key)?.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let asset = PreparedHlsPlaybackAsset {
            authority: authority.clone(),
            root_source: record.root_source.clone()?,
            objects,
        };
        asset.object(asset.root_source()).map(|_| asset)
    }
}
