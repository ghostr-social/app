use ghostr_engine::representation::RepresentationId;
use ghostr_engine::PostId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SegmentedAssetRevision(u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsPreparedAssetAuthority {
    post: PostId,
    representation_id: RepresentationId,
    asset_revision: SegmentedAssetRevision,
}

impl SegmentedAssetRevision {
    pub(super) fn allocate(last: &mut u64) -> Self {
        *last = last
            .checked_add(1)
            .expect("segmented asset revision exhausted");
        Self(*last)
    }
}

impl HlsPreparedAssetAuthority {
    pub(super) fn new(
        post: PostId,
        representation_id: RepresentationId,
        asset_revision: SegmentedAssetRevision,
    ) -> Self {
        Self {
            post,
            representation_id,
            asset_revision,
        }
    }

    pub fn post(&self) -> &PostId {
        &self.post
    }

    pub fn representation_id(&self) -> &RepresentationId {
        &self.representation_id
    }

    pub const fn asset_revision(&self) -> SegmentedAssetRevision {
        self.asset_revision
    }
}

impl super::SegmentedCache {
    pub fn accepts_prepared_authority(&self, authority: &HlsPreparedAssetAuthority) -> bool {
        self.lock()
            .focus
            .get(authority.post())
            .is_some_and(|record| {
                record.snapshot.phase == super::SegmentedPhase::Ready
                    && record.snapshot.authority.as_ref() == Some(authority)
            })
    }
}
