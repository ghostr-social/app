use ghostr_engine::origin_model::NetworkClass;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryNetworkStatus {
    network_class: NetworkClass,
    generation: u64,
}

impl DeliveryNetworkStatus {
    pub const fn new(network_class: NetworkClass, generation: u64) -> Self {
        Self {
            network_class,
            generation,
        }
    }

    pub const fn unavailable() -> Self {
        Self::new(NetworkClass::Unavailable, 0)
    }

    pub(crate) const fn network_class(self) -> NetworkClass {
        self.network_class
    }

    pub(crate) const fn generation(self) -> u64 {
        self.generation
    }

    pub(crate) const fn is_fresher_than(self, previous: Self) -> bool {
        self.generation > previous.generation
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeliveryNetworkStatusReader(Arc<RwLock<DeliveryNetworkStatus>>);

impl DeliveryNetworkStatusReader {
    pub(crate) fn new(status: DeliveryNetworkStatus) -> Self {
        Self(Arc::new(RwLock::new(status)))
    }

    pub(crate) fn network_class(&self) -> NetworkClass {
        self.0
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .network_class()
    }

    pub(crate) fn update(&self, status: DeliveryNetworkStatus) {
        let mut current = self
            .0
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if status.is_fresher_than(*current) {
            *current = status;
        }
    }
}

impl crate::probe::media::ProbeNetwork for DeliveryNetworkStatusReader {
    fn network_class(&self) -> NetworkClass {
        self.network_class()
    }
}
