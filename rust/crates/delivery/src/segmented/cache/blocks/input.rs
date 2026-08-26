use crate::segmented::prepare::PreparedObject;

pub(in crate::segmented) struct StageBlock {
    pub(super) offset: u64,
    pub(super) object: PreparedObject,
    pub(super) complete: bool,
}

impl StageBlock {
    fn new(offset: u64, object: PreparedObject, complete: bool) -> Self {
        Self {
            offset,
            object,
            complete,
        }
    }
}

#[cfg(test)]
#[path = "input_axiom_test.rs"]
pub(crate) mod axiom_test_support;
