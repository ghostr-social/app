#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SegmentedResourceCommitment {
    expected_network_bytes: u64,
    reserved_network_bytes: u64,
}

impl SegmentedResourceCommitment {
    pub(crate) fn new(expected_network_bytes: u64, reserved_network_bytes: u64) -> Option<Self> {
        (expected_network_bytes <= reserved_network_bytes).then_some(Self {
            expected_network_bytes,
            reserved_network_bytes,
        })
    }

    pub(crate) const fn expected_network_bytes(self) -> u64 {
        self.expected_network_bytes
    }

    pub(crate) const fn reserved_network_bytes(self) -> u64 {
        self.reserved_network_bytes
    }
}
