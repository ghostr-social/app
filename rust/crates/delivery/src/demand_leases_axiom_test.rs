use super::*;

impl DemandLeases {
    pub(in super::super) fn len(&self) -> usize {
        self.active.len()
    }
}
