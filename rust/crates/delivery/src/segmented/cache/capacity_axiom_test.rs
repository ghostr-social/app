use super::*;

impl SegmentedCache {
    pub(crate) fn physical_available_bytes(&self) -> u64 {
        physical_available(physical_used(&self.lock()))
    }
}
