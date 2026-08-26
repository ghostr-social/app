use super::*;

impl DeferredRepostBook {
    pub(crate) fn retained_len(&self) -> usize {
        self.entries.len()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.bytes
    }
}
