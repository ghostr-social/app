use super::SingleResponseState;
use crate::partial_range_manifest::RangeManifest;

#[derive(Clone)]
pub(in crate::partial_range_store::single_response) enum StagedPrefix {
    Pending(RangeManifest),
    Readable(RangeManifest),
}

impl StagedPrefix {
    pub(super) const fn manifest(&self) -> &RangeManifest {
        match self {
            Self::Pending(manifest) | Self::Readable(manifest) => manifest,
        }
    }

    fn readable(&self) -> Option<&RangeManifest> {
        match self {
            Self::Pending(_) => None,
            Self::Readable(manifest) => Some(manifest),
        }
    }
}

impl SingleResponseState {
    pub(in crate::partial_range_store::single_response) fn readable_prefix(
        &self,
    ) -> Option<&RangeManifest> {
        self.prefix.as_ref().and_then(StagedPrefix::readable)
    }
}
