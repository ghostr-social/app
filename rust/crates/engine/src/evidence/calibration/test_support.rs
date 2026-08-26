use super::*;

impl FieldReliabilityModel {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("field reliability always serializes")
    }
}
