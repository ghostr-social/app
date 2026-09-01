use super::{
    CacheState, CachedHlsObject, FocusRecord, HlsPreparedAssetAuthority, SegmentedCache,
    SegmentedPhase,
};

#[derive(Clone)]
pub struct PreparedHlsPlaybackAsset {
    authority: HlsPreparedAssetAuthority,
    root_source: String,
    playback_manifest_source: String,
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

    pub fn playback_manifest_source(&self) -> &str {
        &self.playback_manifest_source
    }

    pub fn object(&self, url: &str) -> Option<CachedHlsObject> {
        self.objects
            .iter()
            .find(|known| known.matches(url))
            .map(|known| known.object.clone())
    }

    fn contains_required_manifests(&self) -> bool {
        self.object(self.root_source()).is_some()
            && self.object(self.playback_manifest_source()).is_some()
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
        prepared_record_matches(record, authority, sources)
            .then(|| captured_asset(&state, record, authority))?
    }
}

fn prepared_record_matches(
    record: &FocusRecord,
    authority: &HlsPreparedAssetAuthority,
    sources: &[String],
) -> bool {
    record.snapshot.phase == SegmentedPhase::Ready
        && record.snapshot.authority.as_ref() == Some(authority)
        && record.sources == sources
}

fn captured_asset(
    state: &CacheState,
    record: &FocusRecord,
    authority: &HlsPreparedAssetAuthority,
) -> Option<PreparedHlsPlaybackAsset> {
    let asset = PreparedHlsPlaybackAsset {
        authority: authority.clone(),
        root_source: record.root_source.clone()?,
        playback_manifest_source: record.playback_manifest_source.clone()?,
        objects: captured_objects(state, record)?,
    };
    asset.contains_required_manifests().then_some(asset)
}

fn captured_objects(
    state: &CacheState,
    record: &FocusRecord,
) -> Option<Vec<PreparedHlsPlaybackObject>> {
    record
        .objects
        .iter()
        .map(|key| {
            Some(PreparedHlsPlaybackObject {
                request_url: key.clone(),
                object: state.objects.get(key)?.clone(),
            })
        })
        .collect()
}
