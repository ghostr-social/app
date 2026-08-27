use crate::adaptive::{ActionKind, CandidateSnapshot, MediaLayout};
use crate::origin_model::{MediaClass, OriginRequestProfile, RequestMethod};

pub(super) fn for_action(
    candidate: &CandidateSnapshot,
    action: &ActionKind,
) -> Option<OriginRequestProfile> {
    Some(OriginRequestProfile::new(
        method(action)?,
        bytes(action),
        media(candidate, action),
    ))
}

fn method(action: &ActionKind) -> Option<RequestMethod> {
    match action {
        ActionKind::Head => Some(RequestMethod::Head),
        ActionKind::Prefix(_) => Some(RequestMethod::PrefixGet),
        ActionKind::Tail(_) => Some(RequestMethod::TailGet),
        ActionKind::FetchRange(_) | ActionKind::CacheUpgrade(_) => Some(RequestMethod::RangeGet),
        ActionKind::FetchWhole { .. } => Some(RequestMethod::FullGet),
        ActionKind::HlsBootstrap { stage, .. } if stage.is_manifest() => {
            Some(RequestMethod::ManifestGet)
        }
        ActionKind::HlsBootstrap { .. } => Some(RequestMethod::SegmentGet),
        ActionKind::Promote { .. }
        | ActionKind::Hedge { .. }
        | ActionKind::Transform(_)
        | ActionKind::Cancel(_) => None,
    }
}

fn bytes(action: &ActionKind) -> u64 {
    match action {
        ActionKind::Prefix(range)
        | ActionKind::Tail(range)
        | ActionKind::FetchRange(range)
        | ActionKind::CacheUpgrade(range) => range.len(),
        ActionKind::FetchWhole { maximum_bytes }
        | ActionKind::HlsBootstrap { maximum_bytes, .. } => *maximum_bytes,
        _ => 0,
    }
}

fn media(candidate: &CandidateSnapshot, action: &ActionKind) -> MediaClass {
    if matches!(action, ActionKind::HlsBootstrap { .. }) {
        return MediaClass::Segmented;
    }
    match candidate.layout {
        MediaLayout::Unknown => MediaClass::Unknown,
        MediaLayout::Streamable => MediaClass::ProgressiveMp4,
        MediaLayout::RequiresCompleteFile => MediaClass::WholeObject,
    }
}
