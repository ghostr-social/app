use super::{AdaptiveBufferPolicy, BufferTarget, Duration, MediaConsumption, NetworkConditions};

impl AdaptiveBufferPolicy {
    pub fn target(self, network: NetworkConditions, media: MediaConsumption) -> BufferTarget {
        self.target_for(network, media, Duration::MAX)
    }
}
