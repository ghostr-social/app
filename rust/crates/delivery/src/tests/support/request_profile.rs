use ghostr_engine::origin_model::{
    MediaClass, OriginAttemptProfile, OriginRequestProfile, RequestMethod,
};

pub(crate) fn range_profile(bytes: u64) -> OriginAttemptProfile {
    profile(RequestMethod::RangeGet, bytes, MediaClass::ProgressiveMp4)
}

pub(crate) fn whole_profile(bytes: u64) -> OriginAttemptProfile {
    profile(RequestMethod::FullGet, bytes, MediaClass::WholeObject)
}

fn profile(method: RequestMethod, bytes: u64, media: MediaClass) -> OriginAttemptProfile {
    OriginAttemptProfile::new(OriginRequestProfile::new(method, bytes, media))
}
