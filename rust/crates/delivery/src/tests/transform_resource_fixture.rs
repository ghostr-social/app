use crate::manager::resource_control::{ResourceControl, ResourceEnvironment};
use ghostr_engine::adaptive::ResourceObservation;

pub(super) fn control() -> ResourceControl {
    let target = ResourceObservation::new(1, 1, 100, 1);
    let environment = ResourceEnvironment::new(0, target);
    ResourceControl::new(tokio::time::Instant::now(), environment)
}
