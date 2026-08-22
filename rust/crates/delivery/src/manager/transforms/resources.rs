#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TransformActualResources {
    cpu_ms: u64,
    storage_bytes: u64,
}

impl TransformActualResources {
    pub(super) const fn new(cpu_ms: u64, storage_bytes: u64) -> Self {
        Self {
            cpu_ms,
            storage_bytes,
        }
    }

    pub(crate) const fn cpu_ms(self) -> u64 {
        self.cpu_ms
    }

    pub(crate) const fn storage_bytes(self) -> u64 {
        self.storage_bytes
    }
}
