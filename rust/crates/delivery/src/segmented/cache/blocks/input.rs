use crate::segmented::prepare::PreparedObject;

pub(in crate::segmented) struct StageBlock {
    pub(super) offset: u64,
    pub(super) object: PreparedObject,
    pub(super) complete: bool,
}

impl StageBlock {
    pub(in crate::segmented) fn new(offset: u64, object: PreparedObject, complete: bool) -> Self {
        Self {
            offset,
            object,
            complete,
        }
    }

    #[cfg(test)]
    pub(in crate::segmented) fn partial(offset: u64, object: PreparedObject) -> Self {
        Self::new(offset, object, false)
    }

    #[cfg(test)]
    pub(in crate::segmented) fn complete(offset: u64, object: PreparedObject) -> Self {
        Self::new(offset, object, true)
    }
}
