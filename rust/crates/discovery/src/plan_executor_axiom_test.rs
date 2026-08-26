use super::*;

impl RepostRetryDelta {
    pub(in super::super) fn is_pending(&self) -> bool {
        !self.deferred.is_empty()
    }
}
