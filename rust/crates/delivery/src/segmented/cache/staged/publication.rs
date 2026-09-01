use super::super::{
    objects::insert, CacheState, CachedHlsObject, FocusRecord, HlsPreparedAssetAuthority,
    SegmentedAssetRevision, SegmentedPhase,
};
use crate::segmented::prepare::PreparedComplete;
use ghostr_engine::PostId;

pub(super) fn publish_ready(
    state: &mut CacheState,
    post: &PostId,
    generation: u64,
    playback_manifest: &str,
) -> bool {
    let Some(staged) = take_publishable(state, post, generation, playback_manifest) else {
        return false;
    };
    let keys = publish_objects(state, staged);
    publish_record(state, post, keys, playback_manifest);
    true
}

fn take_publishable(
    state: &mut CacheState,
    post: &PostId,
    generation: u64,
    playback_manifest: &str,
) -> Option<Vec<PreparedComplete>> {
    let record = state.focus.get_mut(post)?;
    publishable(record, generation, playback_manifest).then_some(())?;
    record.reserved_bytes = 0;
    record.assembly_bytes = 0;
    Some(
        core::mem::take(&mut record.staged)
            .into_iter()
            .map(|object| object.into_prepared())
            .collect::<Option<Vec<_>>>()
            .expect("validated complete HLS objects"),
    )
}

fn publishable(record: &FocusRecord, generation: u64, playback_manifest: &str) -> bool {
    record.generation == generation
        && record.preparing.is_none()
        && record.assembly_bytes == 0
        && record.staged.iter().all(|object| object.is_assembled())
        && record
            .staged
            .iter()
            .any(|object| object.request_url() == playback_manifest)
}

fn publish_objects(state: &mut CacheState, staged: Vec<PreparedComplete>) -> Vec<String> {
    staged
        .into_iter()
        .map(|prepared| {
            let key = prepared.object.request_url.clone();
            insert(state, key.clone(), CachedHlsObject::from_prepared(prepared));
            key
        })
        .collect()
}

fn publish_record(
    state: &mut CacheState,
    post: &PostId,
    keys: Vec<String>,
    playback_manifest: &str,
) {
    let revision = SegmentedAssetRevision::allocate(&mut state.last_asset_revision);
    let record = state
        .focus
        .get_mut(post)
        .expect("validated HLS focus record");
    record.objects = keys;
    record.playback_manifest_source = Some(playback_manifest.to_owned());
    record.snapshot.phase = SegmentedPhase::Ready;
    record.snapshot.eta_ms = Some(0);
    record.snapshot.detail = None;
    record.snapshot.authority = Some(HlsPreparedAssetAuthority::new(
        post.clone(),
        record.representation_id.clone(),
        revision,
    ));
}
