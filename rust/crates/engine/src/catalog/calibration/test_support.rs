use super::*;

impl Catalog {
    pub(crate) fn field_reliability(&self) -> &FieldReliabilityModel {
        &self.reliability
    }
}
