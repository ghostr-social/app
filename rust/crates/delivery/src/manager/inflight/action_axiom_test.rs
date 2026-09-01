use super::ChunkAttempt;
use ghostr_engine::origin_model::{
    MediaClass, OriginAttemptProfile, OriginRequestProfile, RequestMethod,
};
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ActionId, ChunkId};

impl ChunkAttempt {
    pub(crate) fn new(chunk: ChunkId, identity: TransferIdentity, id: ActionId) -> Self {
        let profile = test_profile(chunk.range.len());
        Self::new_with_profile(chunk, identity, id, profile)
    }
}

fn test_profile(bytes: u64) -> OriginAttemptProfile {
    OriginAttemptProfile::new(OriginRequestProfile::new(
        RequestMethod::RangeGet,
        bytes,
        MediaClass::ProgressiveMp4,
    ))
}
