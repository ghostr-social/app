use super::*;

impl StageBlock {
    pub(in crate::segmented) fn partial(offset: u64, object: PreparedObject) -> Self {
        Self::new(offset, object, false)
    }
    pub(in crate::segmented) fn complete(offset: u64, object: PreparedObject) -> Self {
        Self::new(offset, object, true)
    }
}
