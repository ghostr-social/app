use super::ClientCapabilityModel;
use crate::client_capability::inference::normalize_record;
use crate::client_capability::types::ClientCapabilityState;

impl ClientCapabilityModel {
    pub(crate) fn from_state(state: ClientCapabilityState) -> Self {
        let ClientCapabilityState {
            generation,
            records,
            revision,
        } = state;
        let Some(generation) = generation else {
            return Self::default();
        };
        let mut model = Self {
            generation: Some(generation),
            generation_confirmed: false,
            ..Self::default()
        };
        for record in records
            .into_iter()
            .filter(|record| record.profile.is_persistent())
            .filter_map(normalize_record)
        {
            model.record(record.profile, record.result);
        }
        model.revision = model.revision.max(revision);
        model
    }
}
