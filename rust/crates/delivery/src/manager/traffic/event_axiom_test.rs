use super::*;

impl TrafficBatch {
    pub(crate) fn events(&self) -> &[TrafficEvent] {
        &self.events
    }
}
