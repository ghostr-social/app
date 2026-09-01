use super::{DecisionPrivacy, RecordedReserveAuthorityOccupancy, ReserveAuthorityOccupancy};

pub(super) fn capture(
    value: &ReserveAuthorityOccupancy,
    privacy: &DecisionPrivacy,
) -> RecordedReserveAuthorityOccupancy {
    RecordedReserveAuthorityOccupancy {
        authority_id: privacy.authority(value.authority.as_str()),
        occupied_request_slots: u64::try_from(value.occupied_request_slots)
            .expect("request occupancy fits the schema-v2 counter"),
        request_width: value.request_width,
    }
}

pub(super) fn sorted(
    mut values: Vec<RecordedReserveAuthorityOccupancy>,
) -> Vec<RecordedReserveAuthorityOccupancy> {
    values.sort_by(|left, right| left.authority_id.cmp(&right.authority_id));
    values
}
