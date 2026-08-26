use super::capacity::{fits, fits_after_reclaim};
use super::staged_object::AssemblySeed;
use super::{SegmentedCache, SegmentedPhase, StageReservation, StagedObject};
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::PostId;
mod commit;
mod types;
pub(crate) use types::{InflightKey, InflightStage};
pub(crate) use types::{StageAdmission, StageFence, StageLease, StageRequest};

impl SegmentedCache {
    pub(crate) fn admit_stage(&self, admission: StageAdmission) -> Option<StageLease> {
        let total = admission.reservation.total_bytes()?;
        let key = InflightKey {
            post: admission.post,
            fence: admission.fence,
        };
        let mut state = self.lock();
        let protected = validate(&state, &key, admission.reservation)?;
        ensure_capacity(&mut state, &key.post, total, protected)?;
        reserve_stage(&mut state, &key, total, admission.eta_ms);
        drop(state);
        self.changed.notify_waiters();
        Some(StageLease {
            cache: self.clone(),
            key: Some(key),
        })
    }

    fn claim_stage_assembly(
        &self,
        key: &InflightKey,
        block: &PreparedObject,
    ) -> Option<AssemblySeed> {
        let mut state = self.lock();
        let record = state.focus.get(&key.post)?;
        let inflight = state.inflight.get(key)?;
        if record.preparing.as_ref() != Some(&key.fence) || inflight.prefix.is_some() {
            return None;
        }
        let index = record
            .staged
            .iter()
            .position(|value| value.request_url() == key.fence.request.url)?;
        let prefix = &record.staged[index];
        if !prefix.matches_identity(block, key.fence.request.offset)
            || block.body.len() as u64 != key.fence.request.block_bytes
        {
            return None;
        }
        let reserved_bytes = inflight.reserved_bytes.checked_add(prefix.len())?;
        let assembly = prefix.assembly_seed();
        let prefix = state.focus.get_mut(&key.post)?.staged.remove(index);
        let inflight = state.inflight.get_mut(key)?;
        inflight.reserved_bytes = reserved_bytes;
        inflight.prefix = Some((index, prefix));
        Some(assembly)
    }

    fn release_stage_lease(&self, key: &InflightKey) {
        let mut state = self.lock();
        let Some(inflight) = state.inflight.remove(key) else {
            return;
        };
        restore_focus_after_release(&mut state, key, inflight);
        drop(state);
        self.changed.notify_waiters();
    }
}

fn restore_focus_after_release(
    state: &mut super::CacheState,
    key: &InflightKey,
    inflight: InflightStage,
) {
    let Some(record) = state.focus.get_mut(&key.post) else {
        return;
    };
    if record.preparing.as_ref() != Some(&key.fence) {
        return;
    }
    record.preparing = None;
    if let Some((index, prefix)) = inflight.prefix {
        record.staged.insert(index.min(record.staged.len()), prefix);
    }
    record.snapshot.bytes_present = record.staged.iter().map(StagedObject::len).sum();
    record.snapshot.phase = SegmentedPhase::Queued;
    record.snapshot.eta_ms = None;
    record.snapshot.detail = None;
}

impl Drop for StageLease {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.cache.release_stage_lease(&key);
        }
    }
}

impl StageLease {
    pub(in crate::segmented) fn claim_assembly(
        &self,
        block: &PreparedObject,
    ) -> Option<AssemblySeed> {
        self.cache.claim_stage_assembly(self.key.as_ref()?, block)
    }
}

fn validate(
    state: &super::CacheState,
    key: &InflightKey,
    reservation: StageReservation,
) -> Option<bool> {
    let record = state.focus.get(&key.post)?;
    record_available(record, key).then_some(())?;
    reservation_matches(reservation, key).then_some(())?;
    (!state.inflight.contains_key(key)).then_some(())?;
    valid_stage_geometry(record, key, reservation).then_some(())?;
    Some(record.protected)
}

fn valid_stage_geometry(
    record: &super::FocusRecord,
    key: &InflightKey,
    reservation: StageReservation,
) -> bool {
    match (key.fence.request.offset, reservation.assembly_bytes) {
        (0, 0) => true,
        (offset, assembly) => {
            let Some(prefix) = record
                .staged
                .iter()
                .find(|value| value.request_url() == key.fence.request.url)
            else {
                return false;
            };
            let geometry = valid_geometry(offset, reservation.block_bytes, assembly);
            prefix.len() == offset && geometry
        }
    }
}

fn ensure_capacity(
    state: &mut super::CacheState,
    post: &PostId,
    total: u64,
    protected: bool,
) -> Option<()> {
    if fits(state, post, total) {
        return Some(());
    }
    (protected && fits_after_reclaim(state, post, total)).then_some(())?;
    super::objects::reclaim_unprotected_ready(state);
    debug_assert!(
        fits(state, post, total),
        "reclaim must make room for a protected HLS stage"
    );
    Some(())
}

fn reserve_stage(state: &mut super::CacheState, key: &InflightKey, total: u64, eta_ms: u64) {
    let record = state.focus.get_mut(&key.post).expect("validated focus");
    record.preparing = Some(key.fence.clone());
    record.snapshot.phase = SegmentedPhase::Preparing;
    record.snapshot.eta_ms = Some(eta_ms);
    record.snapshot.detail = None;
    state.inflight.insert(
        key.clone(),
        InflightStage {
            prefix: None,
            reserved_bytes: total,
        },
    );
}

fn record_available(record: &super::FocusRecord, key: &InflightKey) -> bool {
    (
        record.generation,
        record.snapshot.phase == SegmentedPhase::Ready,
        record.preparing.is_none(),
        record.reserved_bytes,
        record.assembly_bytes,
    ) == (key.fence.generation, false, true, 0, 0)
}

fn reservation_matches(reservation: StageReservation, key: &InflightKey) -> bool {
    (reservation.block_bytes, reservation.block_bytes == 0)
        == (key.fence.request.block_bytes, false)
}

fn valid_geometry(offset: u64, block_bytes: u64, assembly_bytes: u64) -> bool {
    assembly_bytes == 0 || offset.checked_add(block_bytes) == Some(assembly_bytes)
}
