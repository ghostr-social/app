use crate::api::runtime::registry;
use flutter_rust_bridge::frb;
use ghostr_delivery::delivery_events::DeliveryNetworkStatus;
use ghostr_engine::origin_model::NetworkClass;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiDeliveryNetworkClass {
    Unavailable,
    Wifi,
    Cellular,
    Wired,
    Constrained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FfiDeliveryNetworkStatus {
    pub(crate) network_class: FfiDeliveryNetworkClass,
    pub(crate) generation: u64,
}

impl FfiDeliveryNetworkStatus {
    pub const fn unavailable() -> Self {
        Self {
            network_class: FfiDeliveryNetworkClass::Unavailable,
            generation: 0,
        }
    }

    #[frb(ignore)]
    pub const fn wifi(generation: u64) -> Self {
        Self {
            network_class: FfiDeliveryNetworkClass::Wifi,
            generation,
        }
    }
}

impl From<FfiDeliveryNetworkStatus> for DeliveryNetworkStatus {
    fn from(status: FfiDeliveryNetworkStatus) -> Self {
        Self::new(status.network_class.into(), status.generation)
    }
}

impl From<FfiDeliveryNetworkClass> for NetworkClass {
    fn from(network_class: FfiDeliveryNetworkClass) -> Self {
        match network_class {
            FfiDeliveryNetworkClass::Unavailable => Self::Unavailable,
            FfiDeliveryNetworkClass::Wifi => Self::Wifi,
            FfiDeliveryNetworkClass::Cellular => Self::Cellular,
            FfiDeliveryNetworkClass::Wired => Self::Wired,
            FfiDeliveryNetworkClass::Constrained => Self::Constrained,
        }
    }
}

#[frb]
pub fn ffi_set_delivery_network(status: FfiDeliveryNetworkStatus) -> anyhow::Result<bool> {
    let engine = registry::engine()?;
    Ok(engine
        .gateway
        .delivery()
        .update_network_status(status.into()))
}
