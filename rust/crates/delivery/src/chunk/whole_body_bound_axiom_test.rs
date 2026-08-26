use super::*;

impl WholeBodyBoundDiscovered {
    pub(crate) const fn maximum_bytes(self) -> u64 {
        self.maximum_bytes
    }
    pub(crate) const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
}
